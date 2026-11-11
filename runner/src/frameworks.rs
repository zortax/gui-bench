//! The frameworks under test: each one is an app in `apps/<name>` with its own build. An app
//! shows one scenario per run, which the command line names. An edition of a framework, such as
//! `gpui-kit-fast` (the `gpui-fast` feature) or `freya-056a5b4` (a git revision), is its own app
//! directory whose `src` links to the framework's sources.

use std::path::{Path, PathBuf};

/// One framework and the app that represents it.
#[derive(Debug, Clone)]
pub struct Framework {
    /// The short name in results and plots, and the app directory under `apps/`.
    pub name: &'static str,
    /// The app reports frames, handled input and phase spans through fakeland app marks.
    pub marks: bool,
    /// The variants of the stateful scenarios (table and ticker) that the app implements.
    pub variants: &'static [&'static str],
}

const SIGNALS: &[&str] = &["fine", "coarse", "shared"];
const RETAINED: &[&str] = &["fine", "coarse"];
const IMMEDIATE: &[&str] = &["default"];

/// Every known framework.
pub const ALL: &[Framework] = &[
    Framework {
        name: "zgui",
        marks: true,
        variants: SIGNALS,
    },
    Framework {
        name: "gpui-kit",
        marks: false,
        variants: RETAINED,
    },
    Framework {
        name: "gpui-kit-fast",
        marks: false,
        variants: RETAINED,
    },
    Framework {
        name: "freya",
        marks: false,
        variants: SIGNALS,
    },
    Framework {
        name: "freya-056a5b4",
        marks: false,
        variants: SIGNALS,
    },
    Framework {
        name: "vizia",
        marks: false,
        variants: SIGNALS,
    },
    Framework {
        name: "floem",
        marks: false,
        variants: SIGNALS,
    },
    Framework {
        name: "iced",
        marks: false,
        variants: RETAINED,
    },
    Framework {
        name: "egui",
        marks: false,
        variants: IMMEDIATE,
    },
];

impl Framework {
    /// Returns the framework with a name.
    pub fn by_name(name: &str) -> Option<&'static Framework> {
        ALL.iter().find(|f| f.name == name)
    }

    /// Returns the binary name of the app, `<name>-bench`.
    pub fn binary(&self) -> String {
        format!("{}-bench", self.name)
    }

    /// Returns the release binary of the app.
    pub fn binary_path(&self, root: &Path) -> PathBuf {
        root.join("apps")
            .join(self.name)
            .join("target/release")
            .join(self.binary())
    }
}

/// Returns the repository root.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}
