use bench_core::ResourceSample;
use nvml_wrapper::{Nvml, enums::device::UsedGpuMemory};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

fn kb(line: &str) -> Option<u64> {
    line.split_whitespace()
        .nth(1)?
        .parse::<u64>()
        .ok()
        .map(|v| v * 1024)
}

fn value(line: &str) -> Option<u64> {
    line.split_whitespace().nth(1)?.parse().ok()
}

pub fn sample(pid: u32, epoch: Instant) -> ResourceSample {
    let mut out = ResourceSample {
        timestamp_ns: epoch.elapsed().as_nanos() as u64,
        ..ResourceSample::default()
    };
    if let Ok(smaps) = fs::read_to_string(format!("/proc/{pid}/smaps_rollup")) {
        for line in smaps.lines() {
            match line.split(':').next() {
                Some("Rss") => out.rss_bytes = kb(line),
                Some("Pss") => out.pss_bytes = kb(line),
                Some("Private_Clean") => {
                    out.private_bytes = Some(out.private_bytes.unwrap_or(0) + kb(line).unwrap_or(0))
                }
                Some("Private_Dirty") => {
                    out.private_bytes = Some(out.private_bytes.unwrap_or(0) + kb(line).unwrap_or(0))
                }
                Some("Swap") => out.swap_bytes = kb(line),
                _ => {}
            }
        }
    }
    if let Ok(status) = fs::read_to_string(format!("/proc/{pid}/status")) {
        for line in status.lines() {
            match line.split(':').next() {
                Some("VmRSS") if out.rss_bytes.is_none() => out.rss_bytes = kb(line),
                Some("VmHWM") => out.peak_rss_bytes = kb(line),
                Some("Threads") => out.threads = value(line),
                Some("voluntary_ctxt_switches") => out.voluntary_context_switches = value(line),
                Some("nonvoluntary_ctxt_switches") => {
                    out.involuntary_context_switches = value(line)
                }
                _ => {}
            }
        }
    }
    if let Ok(stat) = fs::read_to_string(format!("/proc/{pid}/stat")) {
        // comm may contain spaces; fields following the final ')' have stable positions.
        if let Some(rest) = stat.rsplit_once(')').map(|(_, rest)| rest.trim()) {
            let fields: Vec<_> = rest.split_whitespace().collect();
            out.cpu_user_ticks = fields.get(11).and_then(|v| v.parse().ok());
            out.cpu_system_ticks = fields.get(12).and_then(|v| v.parse().ok());
        }
    }
    sample_drm_fdinfo(pid, &mut out);
    if out.gpu_resident_bytes.is_none() {
        sample_nvml(pid, &mut out);
    }
    out
}

fn sample_nvml(pid: u32, out: &mut ResourceSample) {
    static NVML: OnceLock<Option<Nvml>> = OnceLock::new();
    let Some(nvml) = NVML.get_or_init(|| Nvml::init().ok()) else {
        return;
    };
    let Ok(count) = nvml.device_count() else {
        return;
    };
    let mut bytes = 0u64;
    let mut found = false;
    for index in 0..count {
        let Ok(device) = nvml.device_by_index(index) else {
            continue;
        };
        let Ok(processes) = device.running_graphics_processes() else {
            continue;
        };
        for process in processes.into_iter().filter(|process| process.pid == pid) {
            if let UsedGpuMemory::Used(value) = process.used_gpu_memory {
                bytes = bytes.saturating_add(value);
                found = true;
            }
        }
    }
    if found {
        out.gpu_resident_bytes = Some(bytes);
        out.gpu_source = Some("nvml_graphics_process".into());
    }
}

fn sample_drm_fdinfo(pid: u32, out: &mut ResourceSample) {
    let dir = PathBuf::from(format!("/proc/{pid}/fdinfo"));
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut resident = 0u64;
    let mut shared = 0u64;
    let mut found = false;
    for entry in entries.flatten() {
        let Ok(text) = fs::read_to_string(entry.path()) else {
            continue;
        };
        if !text.contains("drm-") {
            continue;
        }
        for line in text.lines() {
            let key = line.split(':').next().unwrap_or_default();
            let bytes = parse_drm_bytes(line).unwrap_or(0);
            if key.starts_with("drm-resident-") || key == "drm-memory-vram" {
                resident = resident.saturating_add(bytes);
                found = true;
            } else if key.starts_with("drm-shared-") || key == "drm-memory-gtt" {
                shared = shared.saturating_add(bytes);
                found = true;
            }
        }
    }
    if found {
        out.gpu_resident_bytes = Some(resident);
        out.gpu_shared_bytes = Some(shared);
        out.gpu_source = Some("drm_fdinfo".into());
    }
}

fn parse_drm_bytes(line: &str) -> Option<u64> {
    let mut parts = line.split_whitespace();
    let _key = parts.next()?;
    let number = parts.next()?.parse::<u64>().ok()?;
    match parts.next().unwrap_or("B") {
        "KiB" | "kB" => Some(number * 1024),
        "MiB" => Some(number * 1024 * 1024),
        "GiB" => Some(number * 1024 * 1024 * 1024),
        _ => Some(number),
    }
}
