//! Framework-neutral scenarios, adapter protocol, and result schema.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{self, Write};

pub const SCHEMA_VERSION: u32 = 1;
pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Framework {
    #[default]
    Zgui,
    Gpui,
}

impl Framework {
    pub const ALL: [Self; 2] = [Self::Zgui, Self::Gpui];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Zgui => "zgui",
            Self::Gpui => "gpui",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementMode {
    #[default]
    Headline,
    Diagnostic,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Workload {
    ColdStart,
    Idle,
    PaintUpdate,
    LayoutUpdate,
    TextUpdate,
    StructuralUpdate,
    InputLatency,
    ResizeStep,
    ResizeStorm,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Topology {
    Flat,
    Wide,
    Balanced,
    Deep,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StyleComplexity {
    Minimal,
    Decorated,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Scenario {
    pub id: String,
    pub workload: Workload,
    pub nodes: usize,
    pub topology: Topology,
    pub style: StyleComplexity,
    pub update_fraction: f64,
    pub warmup_frames: u32,
    pub sample_frames: u32,
    pub width: u32,
    pub height: u32,
    pub seed: u64,
}

impl Scenario {
    pub fn validate(&self) -> Result<(), String> {
        if self.nodes == 0 {
            return Err("nodes must be non-zero".into());
        }
        if !(0.0..=1.0).contains(&self.update_fraction) {
            return Err("update_fraction must be in [0, 1]".into());
        }
        if self.sample_frames == 0 {
            return Err("sample_frames must be non-zero".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    Smoke,
    Standard,
    Full,
}

pub fn scenarios(preset: Preset) -> Vec<Scenario> {
    let (warmup, frames, sizes, fractions): (u32, u32, &[usize], &[f64]) = match preset {
        Preset::Smoke => (5, 30, &[64], &[1.0]),
        Preset::Standard => (30, 180, &[64, 1_024, 8_192], &[0.01, 0.25, 1.0]),
        Preset::Full => (
            60,
            600,
            &[16, 64, 256, 1_024, 4_096, 16_384],
            &[0.001, 0.01, 0.1, 1.0],
        ),
    };
    let mut out = Vec::new();
    let mut add = |workload, nodes, topology, style, update_fraction: f64| {
        let id = format!(
            "{:?}-n{}-{:?}-{:?}-u{}",
            workload,
            nodes,
            topology,
            style,
            (update_fraction * 1_000.0).round() as u32
        )
        .to_ascii_lowercase();
        out.push(Scenario {
            id,
            workload,
            nodes,
            topology,
            style,
            update_fraction,
            warmup_frames: warmup,
            sample_frames: frames,
            width: 1_200,
            height: 800,
            seed: 0x5eed_f00d,
        });
    };

    // Cold/idle establish fixed overhead. The matrix then separates paint, layout,
    // text shaping, reconciliation, topology, styling, and resize behavior.
    add(
        Workload::ColdStart,
        sizes[0],
        Topology::Flat,
        StyleComplexity::Minimal,
        0.0,
    );
    add(
        Workload::Idle,
        sizes[0],
        Topology::Flat,
        StyleComplexity::Minimal,
        0.0,
    );
    for &nodes in sizes {
        for &fraction in fractions {
            add(
                Workload::PaintUpdate,
                nodes,
                Topology::Flat,
                StyleComplexity::Minimal,
                fraction,
            );
            add(
                Workload::LayoutUpdate,
                nodes,
                Topology::Balanced,
                StyleComplexity::Minimal,
                fraction,
            );
            add(
                Workload::TextUpdate,
                nodes,
                Topology::Wide,
                StyleComplexity::Minimal,
                fraction,
            );
        }
        add(
            Workload::StructuralUpdate,
            nodes,
            Topology::Wide,
            StyleComplexity::Minimal,
            0.1,
        );
        add(
            Workload::PaintUpdate,
            nodes,
            Topology::Balanced,
            StyleComplexity::Decorated,
            1.0,
        );
        if nodes <= 1_024 {
            add(
                Workload::LayoutUpdate,
                nodes,
                Topology::Deep,
                StyleComplexity::Minimal,
                1.0,
            );
        }
    }
    let largest = *sizes.last().unwrap_or(&64);
    add(
        Workload::InputLatency,
        largest,
        Topology::Balanced,
        StyleComplexity::Decorated,
        0.01,
    );
    add(
        Workload::ResizeStep,
        largest,
        Topology::Balanced,
        StyleComplexity::Decorated,
        1.0,
    );
    add(
        Workload::ResizeStorm,
        largest,
        Topology::Balanced,
        StyleComplexity::Decorated,
        1.0,
    );
    out
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogicalNode {
    pub id: usize,
    pub parent: Option<usize>,
    pub depth: usize,
    pub label: String,
}

pub fn logical_nodes(spec: &Scenario) -> Vec<LogicalNode> {
    let mut nodes: Vec<LogicalNode> = Vec::with_capacity(spec.nodes);
    for id in 0..spec.nodes {
        let parent = match spec.topology {
            Topology::Flat => None,
            Topology::Wide => (id > 0).then_some(0),
            Topology::Balanced => {
                if id > 0 {
                    Some((id - 1) / 8)
                } else {
                    None
                }
            }
            Topology::Deep => (id > 0).then_some(id - 1),
        };
        let depth = parent.map(|p| nodes[p].depth + 1).unwrap_or(0);
        nodes.push(LogicalNode {
            id,
            parent,
            depth,
            label: format!("item {id:05}"),
        });
    }
    nodes
}

/// A stable selection shared by adapters. Sequential access is intentional: it
/// avoids measuring PRNG differences and yields deterministic sparse updates.
pub fn affected_indices(spec: &Scenario, tick: u64) -> Vec<usize> {
    let count = ((spec.nodes as f64 * spec.update_fraction).ceil() as usize)
        .max(usize::from(spec.update_fraction > 0.0))
        .min(spec.nodes);
    let start = (tick as usize).wrapping_mul(2_654_435_761usize) % spec.nodes;
    (0..count).map(|i| (start + i) % spec.nodes).collect()
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PipelineSample {
    /// CPU-side view/DOM construction or reconciliation.
    pub view_build_ns: Option<u64>,
    pub style_ns: Option<u64>,
    pub layout_ns: Option<u64>,
    pub prepaint_ns: Option<u64>,
    pub paint_ns: Option<u64>,
    pub gpu_prepare_ns: Option<u64>,
    pub command_encode_ns: Option<u64>,
    pub queue_submit_ns: Option<u64>,
    pub present_ns: Option<u64>,
    /// True only for actual device timestamp queries; false means CPU submission timing.
    pub device_gpu_timestamps: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FrameworkMemory {
    pub cpu_bytes: Option<u64>,
    pub gpu_bytes: Option<u64>,
    pub atlas_bytes: Option<u64>,
    pub details: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FrameSample {
    pub sequence: u64,
    pub phase: SamplePhase,
    pub timestamp_ns: u64,
    pub frame_time_ns: Option<u64>,
    pub stimulus_latency_ns: Option<u64>,
    pub observed_width: Option<u32>,
    pub observed_height: Option<u32>,
    pub pipeline: PipelineSample,
    pub framework_memory: Option<FrameworkMemory>,
    pub valid: bool,
    pub note: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SamplePhase {
    Startup,
    Warmup,
    Measurement,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdapterEvent {
    Hello {
        protocol_version: u32,
        framework: Framework,
        framework_version: String,
        mode: MeasurementMode,
        pid: u32,
        capabilities: Vec<String>,
    },
    Ready,
    Frame {
        sample: FrameSample,
    },
    Complete {
        frames: u64,
    },
    Error {
        message: String,
    },
}

pub fn emit_event(event: &AdapterEvent) -> io::Result<()> {
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    serde_json::to_writer(&mut lock, event)?;
    lock.write_all(b"\n")?;
    lock.flush()
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ResourceSample {
    pub timestamp_ns: u64,
    pub rss_bytes: Option<u64>,
    pub pss_bytes: Option<u64>,
    pub private_bytes: Option<u64>,
    pub swap_bytes: Option<u64>,
    pub peak_rss_bytes: Option<u64>,
    pub cpu_user_ticks: Option<u64>,
    pub cpu_system_ticks: Option<u64>,
    pub voluntary_context_switches: Option<u64>,
    pub involuntary_context_switches: Option<u64>,
    pub threads: Option<u64>,
    pub gpu_resident_bytes: Option<u64>,
    pub gpu_shared_bytes: Option<u64>,
    pub gpu_source: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConditionResult {
    pub framework: Framework,
    pub mode: MeasurementMode,
    pub repetition: u32,
    pub scenario: Scenario,
    pub adapter: AdapterMetadata,
    pub frames: Vec<FrameSample>,
    pub resources: Vec<ResourceSample>,
    pub wall_time_ns: u64,
    pub status: RunStatus,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AdapterMetadata {
    pub version: String,
    pub executable: String,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Complete,
    Failed { message: String },
    TimedOut,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HostMetadata {
    pub os: String,
    pub kernel: String,
    pub architecture: String,
    pub cpu: String,
    pub gpu: Vec<String>,
    pub display_server: String,
    pub rustc: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SourceMetadata {
    pub path: String,
    pub revision: Option<String>,
    pub dirty: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunDocument {
    pub schema_version: u32,
    pub run_id: String,
    pub started_at_unix_ms: u64,
    pub command: Vec<String>,
    pub preset: Preset,
    pub host: HostMetadata,
    pub sources: BTreeMap<String, SourceMetadata>,
    pub conditions: Vec<ConditionResult>,
    pub notes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balanced_tree_is_stable_and_bounded() {
        let spec = scenarios(Preset::Smoke).into_iter().next().unwrap();
        let mut spec = Scenario {
            nodes: 1_000,
            topology: Topology::Balanced,
            ..spec
        };
        let a = logical_nodes(&spec);
        let b = logical_nodes(&spec);
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            serde_json::to_vec(&b).unwrap()
        );
        assert!(a.iter().all(|n| n.parent.is_none_or(|p| p < n.id)));
        spec.update_fraction = 0.01;
        assert_eq!(affected_indices(&spec, 3).len(), 10);
    }

    #[test]
    fn scenario_ids_are_unique() {
        for preset in [Preset::Smoke, Preset::Standard, Preset::Full] {
            let cases = scenarios(preset);
            let ids: std::collections::BTreeSet<_> = cases.iter().map(|s| &s.id).collect();
            assert_eq!(ids.len(), cases.len());
        }
    }
}
