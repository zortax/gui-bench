//! The frameworks under test: each one is a rows app in `apps/` with its own build.

use std::path::{Path, PathBuf};

/// One framework and the rows app that represents it.
#[derive(Debug, Clone)]
pub struct Framework {
    /// The short name in results and plots.
    pub name: &'static str,
    /// The app directory under `apps/`.
    pub dir: &'static str,
    /// The binary name in the app's release target.
    pub binary: &'static str,
    /// The app reports frames, handled input and phase spans through fakeland app marks.
    pub marks: bool,
}

/// Every known framework.
pub const ALL: &[Framework] = &[
    Framework {
        name: "zgui",
        dir: "zgui-rows",
        binary: "zgui-rows",
        marks: true,
    },
    Framework {
        name: "gpui-kit",
        dir: "gpui-kit-rows",
        binary: "gpui-kit-rows",
        marks: false,
    },
];

impl Framework {
    /// Returns the framework with a name.
    pub fn by_name(name: &str) -> Option<&'static Framework> {
        ALL.iter().find(|f| f.name == name)
    }

    /// Returns the release binary of the app.
    pub fn binary_path(&self, root: &Path) -> PathBuf {
        root.join("apps")
            .join(self.dir)
            .join("target/release")
            .join(self.binary)
    }
}

/// Returns the repository root.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}
