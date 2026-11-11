//! The errors of the data layer.

use std::path::PathBuf;

/// An error while reading a run or one of its files.
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("no run {0}")]
    NoRun(String),
    #[error("run {0} has a format the explorer cannot read: {1}")]
    Unsupported(String, String),
    #[error("cannot load the run: {0}")]
    Load(String),
    #[error("no cell {0} in the run")]
    NoCell(u32),
    #[error("the cell has no report {0}")]
    NoReport(String),
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot parse {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
}

/// Reads and parses a JSON file.
pub(crate) fn read_json<T: serde::de::DeserializeOwned>(
    path: &std::path::Path,
) -> Result<T, DataError> {
    let bytes = std::fs::read(path).map_err(|source| DataError::Io {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| DataError::Json {
        path: path.to_owned(),
        source,
    })
}
