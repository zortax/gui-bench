mod report;
mod resource;

use anyhow::{Context, Result, bail};
use bench_core::{
    AdapterEvent, AdapterMetadata, ConditionResult, Framework, HostMetadata, MeasurementMode,
    Preset, RunDocument, RunStatus, SCHEMA_VERSION, SourceMetadata,
};
use clap::{Parser, Subcommand, ValueEnum};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

#[derive(Parser)]
#[command(about = "Cross-framework native GUI benchmark harness")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Preflight,
    List {
        #[arg(long, value_enum, default_value = "standard")]
        preset: PresetArg,
    },
    Run {
        #[arg(long, value_enum, default_value = "standard")]
        preset: PresetArg,
        #[arg(long, value_enum)]
        framework: Vec<FrameworkArg>,
        #[arg(long)]
        diagnostic: bool,
        #[arg(long, default_value_t = 3)]
        repetitions: u32,
        #[arg(long)]
        scenario: Option<String>,
        #[arg(long, default_value = "results/latest.json")]
        output: PathBuf,
        #[arg(long)]
        no_build: bool,
    },
    Report {
        #[arg(default_value = "results/latest.json")]
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum PresetArg {
    Smoke,
    Standard,
    Full,
}
impl From<PresetArg> for Preset {
    fn from(v: PresetArg) -> Self {
        match v {
            PresetArg::Smoke => Self::Smoke,
            PresetArg::Standard => Self::Standard,
            PresetArg::Full => Self::Full,
        }
    }
}
#[derive(Clone, Copy, ValueEnum)]
enum FrameworkArg {
    Zgui,
    Gpui,
}
impl From<FrameworkArg> for Framework {
    fn from(v: FrameworkArg) -> Self {
        match v {
            FrameworkArg::Zgui => Self::Zgui,
            FrameworkArg::Gpui => Self::Gpui,
        }
    }
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Preflight => preflight(),
        Commands::List { preset } => {
            for s in bench_core::scenarios(preset.into()) {
                println!(
                    "{}\t{:?}\t{} nodes\t{:?}\t{:?}\t{:.1}%",
                    s.id,
                    s.workload,
                    s.nodes,
                    s.topology,
                    s.style,
                    s.update_fraction * 100.0
                );
            }
            Ok(())
        }
        Commands::Run {
            preset,
            framework,
            diagnostic,
            repetitions,
            scenario,
            output,
            no_build,
        } => run(
            preset.into(),
            framework,
            diagnostic,
            repetitions,
            scenario.as_deref(),
            &output,
            no_build,
        ),
        Commands::Report { input, output } => {
            let doc: RunDocument = serde_json::from_slice(&fs::read(&input)?)?;
            let output = output.unwrap_or_else(|| input.with_extension("html"));
            report::render(&doc, &output)?;
            println!("wrote {}", output.display());
            Ok(())
        }
    }
}

fn preflight() -> Result<()> {
    let mut failures = Vec::new();
    let root = workspace_root();
    for path in [
        root.join("../zgui/crates/zgui/Cargo.toml"),
        root.join("vendor/gpui-ce/crates/gpui/Cargo.toml"),
        root.join("assets/echarts.min.js"),
    ] {
        if path.exists() {
            println!("ok       {}", path.display());
        } else {
            println!("missing  {}", path.display());
            failures.push(path);
        }
    }
    for var in ["WAYLAND_DISPLAY", "DISPLAY"] {
        println!(
            "{var:<16}{}",
            std::env::var(var).unwrap_or_else(|_| "(unset)".into())
        );
    }
    println!("DRM fdinfo      per-process support is detected during each run");
    if !failures.is_empty() {
        bail!("preflight failed: {} missing item(s)", failures.len())
    } else {
        Ok(())
    }
}

fn run(
    preset: Preset,
    requested: Vec<FrameworkArg>,
    diagnostic: bool,
    repetitions: u32,
    only: Option<&str>,
    output: &Path,
    no_build: bool,
) -> Result<()> {
    if !no_build {
        // Build in separate Cargo invocations. The two upstream projects activate
        // mutually incompatible feature combinations in a unified dependency
        // graph, while the adapters themselves are separate processes.
        for package in ["bench-zgui", "bench-gpui"] {
            let status = Command::new("cargo")
                .current_dir(workspace_root())
                .args(["build", "--profile", "bench-run", "-p", package])
                .status()?;
            if !status.success() {
                bail!("{package} adapter build failed")
            }
        }
    }
    let frameworks: Vec<Framework> = if requested.is_empty() {
        Framework::ALL.to_vec()
    } else {
        requested.into_iter().map(Into::into).collect()
    };
    let mut cases = bench_core::scenarios(preset);
    if let Some(filter) = only {
        cases.retain(|s| s.id.contains(filter));
    }
    if cases.is_empty() {
        bail!("no scenarios matched")
    }
    let modes = if diagnostic {
        vec![MeasurementMode::Headline, MeasurementMode::Diagnostic]
    } else {
        vec![MeasurementMode::Headline]
    };
    let now_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
    let mut doc = RunDocument {
        schema_version: SCHEMA_VERSION,
        run_id: format!("{}-{now_ms}", hostname()),
        started_at_unix_ms: now_ms,
        command: std::env::args().collect(),
        preset,
        host: host_metadata(),
        sources: source_metadata(),
        conditions: Vec::new(),
        notes: vec![
            "Headline and diagnostic passes are separate to expose instrumentation overhead."
                .into(),
        ],
    };
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?
    }
    let total = frameworks.len() * modes.len() * cases.len() * repetitions as usize;
    let mut n = 0;
    for framework in frameworks {
        for &mode in &modes {
            for spec in &cases {
                for repetition in 0..repetitions {
                    n += 1;
                    eprintln!(
                        "[{n}/{total}] {} {:?} {} repetition {}",
                        framework.as_str(),
                        mode,
                        spec.id,
                        repetition + 1
                    );
                    doc.conditions
                        .push(run_condition(framework, mode, spec, repetition)?);
                    write_document(output, &doc)?;
                }
            }
        }
    }
    let html = output.with_extension("html");
    report::render(&doc, &html)?;
    println!("wrote {} and {}", output.display(), html.display());
    Ok(())
}

fn run_condition(
    framework: Framework,
    mode: MeasurementMode,
    spec: &bench_core::Scenario,
    repetition: u32,
) -> Result<ConditionResult> {
    let exe = workspace_root()
        .join("target/bench-run")
        .join(match framework {
            Framework::Zgui => "bench-zgui",
            Framework::Gpui => "bench-gpui",
        });
    let mut child = Command::new(&exe)
        .arg("--scenario")
        .arg(serde_json::to_string(spec)?)
        .arg("--mode")
        .arg(match mode {
            MeasurementMode::Headline => "headline",
            MeasurementMode::Diagnostic => "diagnostic",
        })
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("spawn {}", exe.display()))?;
    let pid = child.id();
    let started = Instant::now();
    let done = Arc::new(AtomicBool::new(false));
    let sampler_done = done.clone();
    let (sample_tx, sample_rx) = mpsc::channel();
    thread::spawn(move || {
        while !sampler_done.load(Ordering::Relaxed) {
            let _ = sample_tx.send(resource::sample(pid, started));
            thread::sleep(Duration::from_millis(50));
        }
    });
    let stdout = child.stdout.take().unwrap();
    let (event_tx, event_rx) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            match line {
                Ok(line) => {
                    let parsed = serde_json::from_str::<AdapterEvent>(&line)
                        .map_err(|e| format!("{e}: {line}"));
                    if event_tx.send(parsed).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let _ = event_tx.send(Err(e.to_string()));
                    break;
                }
            }
        }
    });
    let mut frames = Vec::new();
    let mut warnings = Vec::new();
    let mut meta = AdapterMetadata {
        executable: exe.display().to_string(),
        ..Default::default()
    };
    let mut status = RunStatus::TimedOut;
    let timeout = Duration::from_secs((spec.warmup_frames + spec.sample_frames) as u64 / 10 + 45);
    let first = event_rx
        .recv_timeout(Duration::from_secs(30))
        .context("adapter did not send hello")?
        .map_err(anyhow::Error::msg)?;
    match first {
        AdapterEvent::Hello {
            protocol_version,
            framework: actual,
            framework_version,
            capabilities,
            ..
        } => {
            if protocol_version != bench_core::PROTOCOL_VERSION {
                bail!("protocol mismatch")
            };
            if actual != framework {
                bail!("adapter framework mismatch")
            };
            meta.version = framework_version;
            meta.capabilities = capabilities
        }
        other => bail!("expected hello, got {other:?}"),
    }
    child.stdin.as_mut().unwrap().write_all(b"start\n")?;
    while started.elapsed() < timeout {
        match event_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Ok(AdapterEvent::Frame { sample })) => frames.push(sample),
            Ok(Ok(AdapterEvent::Complete { .. })) => {
                status = RunStatus::Complete;
                break;
            }
            Ok(Ok(AdapterEvent::Error { message })) => {
                status = RunStatus::Failed { message };
                break;
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                status = RunStatus::Failed { message: e };
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(exit) = child.try_wait()? {
                    status = RunStatus::Failed {
                        message: format!("adapter exited early: {exit}"),
                    };
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                status = RunStatus::Failed {
                    message: "adapter output closed".into(),
                };
                break;
            }
        }
    }
    done.store(true, Ordering::Relaxed);
    if child.try_wait()?.is_none() {
        let _ = child.kill();
    }
    let _ = child.wait();
    let resources: Vec<_> = sample_rx.try_iter().collect();
    if resources.iter().all(|r| r.gpu_resident_bytes.is_none()) {
        warnings.push("per-process VRAM unavailable from DRM fdinfo and NVML".into())
    }
    let observed: std::collections::BTreeSet<_> = frames
        .iter()
        .filter(|frame| frame.phase == bench_core::SamplePhase::Measurement)
        .filter_map(|frame| Some((frame.observed_width?, frame.observed_height?)))
        .collect();
    if !matches!(
        spec.workload,
        bench_core::Workload::ResizeStep | bench_core::Workload::ResizeStorm
    ) && observed
        .iter()
        .any(|&(width, height)| width != spec.width || height != spec.height)
    {
        warnings.push(format!(
            "compositor chose {:?} instead of requested {}x{}; use the observed size for comparisons",
            observed, spec.width, spec.height
        ));
    }
    if matches!(
        spec.workload,
        bench_core::Workload::ResizeStep | bench_core::Workload::ResizeStorm
    ) && observed.len() < 2
    {
        warnings.push("compositor did not grant multiple observed window sizes; resize result is not valid for slope/latency conclusions".into());
    }
    Ok(ConditionResult {
        framework,
        mode,
        repetition,
        scenario: spec.clone(),
        adapter: meta,
        frames,
        resources,
        wall_time_ns: started.elapsed().as_nanos() as u64,
        status,
        warnings,
    })
}

fn write_document(path: &Path, doc: &RunDocument) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(doc)?)?;
    fs::rename(tmp, path)?;
    Ok(())
}
fn cmd(args: &[&str]) -> String {
    Command::new(args[0])
        .args(&args[1..])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}
fn hostname() -> String {
    cmd(&["hostname"]).replace(|c: char| !c.is_ascii_alphanumeric(), "-")
}
fn host_metadata() -> HostMetadata {
    let lspci = cmd(&["lspci"]);
    HostMetadata {
        os: std::env::consts::OS.into(),
        kernel: cmd(&["uname", "-r"]),
        architecture: std::env::consts::ARCH.into(),
        cpu: fs::read_to_string("/proc/cpuinfo")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("model name"))
                    .and_then(|l| l.split_once(':'))
                    .map(|x| x.1.trim().into())
            })
            .unwrap_or_default(),
        gpu: lspci
            .lines()
            .filter(|l| {
                let l = l.to_ascii_lowercase();
                l.contains("vga") || l.contains("3d controller") || l.contains("display controller")
            })
            .map(str::to_string)
            .collect(),
        display_server: if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            "wayland"
        } else if std::env::var_os("DISPLAY").is_some() {
            "x11"
        } else {
            "unknown"
        }
        .into(),
        rustc: cmd(&["rustc", "-Vv"]),
    }
}
fn source(path: &Path) -> SourceMetadata {
    let shown = path.display().to_string();
    let rev = Command::new("git")
        .args(["-C", &shown, "rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().into());
    let dirty = Command::new("git")
        .args(["-C", &shown, "status", "--porcelain"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| !o.stdout.is_empty());
    SourceMetadata {
        path: shown,
        revision: rev,
        dirty,
    }
}
fn source_metadata() -> BTreeMap<String, SourceMetadata> {
    let root = workspace_root();
    BTreeMap::from([
        ("zgui".into(), source(&root.join("../zgui"))),
        (
            "gpui-ce-vendored-from".into(),
            source(&root.join("../gpui-ce")),
        ),
    ])
}
