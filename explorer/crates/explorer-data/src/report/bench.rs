//! The fakeland bench report, reduced to what the run store keeps: the environment, the output,
//! the validity, the metric summaries of every section and the pooled latency samples. Serde
//! skips every other field.

use explorer_core::api::{Better, CheckStatus, Stat};
use serde::Deserialize;

/// A bench report (`format_version` 1).
#[derive(Debug, Clone, Deserialize)]
pub struct BenchReport {
    pub format_version: Option<u32>,
    pub params: Option<serde_json::Value>,
    pub environment: Option<Environment>,
    pub config: Option<Config>,
    pub validity: Option<Validity>,
    #[serde(default)]
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Environment {
    pub gpu: String,
    pub driver: String,
    pub kernel: String,
    pub cpu: String,
    pub governor: String,
    pub renderer: String,
    pub hostname: String,
    pub fakeland_version: String,
}

impl Environment {
    pub fn to_api(&self) -> explorer_core::api::Environment {
        explorer_core::api::Environment {
            cpu: self.cpu.clone(),
            gpu: self.gpu.clone(),
            driver: self.driver.clone(),
            kernel: self.kernel.clone(),
            governor: self.governor.clone(),
            renderer: self.renderer.clone(),
            hostname: self.hostname.clone(),
            fakeland_version: self.fakeland_version.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub outputs: Vec<OutputState>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputState {
    pub name: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Validity {
    pub valid: bool,
    #[serde(default)]
    pub checks: Vec<CheckRaw>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckRaw {
    pub name: String,
    pub status: String,
}

impl CheckRaw {
    pub fn status(&self) -> CheckStatus {
        CheckStatus::parse(&self.status)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section {
    pub name: String,
    #[serde(default)]
    pub metrics: Vec<MetricRaw>,
    #[serde(default)]
    pub pooled: Vec<Pooled>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricRaw {
    pub name: String,
    pub variant: Option<serde_json::Value>,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub better: String,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub stddev: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub ci: Option<Ci>,
    #[serde(default)]
    pub values: Vec<Option<f64>>,
}

impl MetricRaw {
    /// Returns the statistic over iterations, or `None` without a median.
    pub fn stat(&self) -> Option<Stat> {
        let median = self.median?;
        let values: Vec<f64> = self.values.iter().flatten().copied().collect();
        Some(Stat {
            median,
            mean: self.mean.unwrap_or(median),
            stddev: self.stddev.unwrap_or(0.0),
            min: self.min.unwrap_or(median),
            max: self.max.unwrap_or(median),
            ci: self.ci.as_ref().map(|c| (c.lo, c.hi)),
            values,
        })
    }

    pub fn better(&self) -> Better {
        Better::parse(&self.better)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ci {
    pub lo: f64,
    pub hi: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pooled {
    pub variant: Option<serde_json::Value>,
    pub report: Option<PooledReport>,
}

/// The pooled latency report: every repeat's sample and the CI of the median.
#[derive(Debug, Clone, Deserialize)]
pub struct PooledReport {
    #[serde(default)]
    pub samples: Vec<SampleRaw>,
    pub total_median_ci: Option<Ci>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SampleRaw {
    pub total_ms: Option<f64>,
    #[serde(default)]
    pub timed_out: bool,
}

impl BenchReport {
    /// Returns a section by name.
    pub fn section(&self, name: &str) -> Option<&Section> {
        self.sections.iter().find(|s| s.name == name)
    }

    /// Returns the `repeats` parameter of the preset.
    pub fn repeats(&self) -> Option<f64> {
        self.params.as_ref()?.get("repeats")?.as_f64()
    }
}
