//! The frameworks under test: each one is an app in `apps/<name>` with its own build. `tauri` is
//! the one app that is no native Rust UI: a Tauri shell around a SolidJS page in the system web
//! view, as a reference for the web stack. An app
//! shows one scenario per run, which the command line names. An edition of a framework is another
//! build of the same sources. `gpui-kit-tuned` and `iced-tuned` build their app with the `tuned`
//! feature into `target/tuned`: the app then opts into the caching that the framework offers
//! (`cached` views, `lazy` widgets), which the plain build leaves out. `dioxus-skia` builds
//! `apps/dioxus` with the `skia` feature into `target/skia`; `gpui-kit-fast` and `freya-056a5b4` need dependency versions that cannot share
//! a lock file with their framework, so each is its own app directory whose `src` links to the
//! framework's sources.

use std::path::{Path, PathBuf};

/// One framework and the app that represents it.
#[derive(Debug, Clone)]
pub struct Framework {
    /// The short name in results and plots.
    pub name: &'static str,
    /// The app directory under `apps/`; its binary is `<app>-bench`.
    pub app: &'static str,
    /// The crate that stands for the framework in the app's `Cargo.lock`, such as `eframe`.
    pub package: &'static str,
    /// The target directory under the app's `target/` for a build with other features, such as
    /// `skia`, or empty for the default build.
    pub build: &'static str,
    /// The app reports frames, handled input and phase spans through fakeland app marks.
    pub marks: bool,
    /// The framework runs without `--frameworks`. Editions that compare versions or renderers
    /// of a framework run only by name.
    pub default: bool,
    /// The parts of the scenarios that the app builds by hand or around a defect, because the
    /// framework has no working part of its own: `(part, note)`. Results mark them.
    pub flags: &'static [(&'static str, &'static str)],
}

const NO_FLAGS: &[(&str, &str)] = &[];
const EGUI: &[(&str, &str)] = &[(
    "switch",
    "no switch widget: the app paints the toggle of the egui demo",
)];
const ICED: &[(&str, &str)] = &[
    (
        "list",
        "no virtual list: the app builds the rows of the viewport from the scroll offset",
    ),
    (
        "menu",
        "no menu or popover: the app pins a container of buttons in a stack over the window",
    ),
    (
        "spinner",
        "no transform animation: the app redraws a turned square on a canvas every frame",
    ),
];
const FREYA: &[(&str, &str)] = &[(
    "menu",
    "the menu has no anchor: the app places it in an absolute box under the button",
)];
const GPUI: &[(&str, &str)] = &[(
    "spinner",
    "only svg elements rotate: the square is an inline svg, as in the kit's own spinner",
)];
const VIZIA: &[(&str, &str)] = &[(
    "spinner",
    "no repeating animation: a 16 ms timer turns the square, at about 62 Hz at any rate",
)];
const FLOEM: &[(&str, &str)] = &[
    (
        "menu",
        "no styleable menu: the app adds a stack of items as an overlay under the button",
    ),
    (
        "spinner",
        "a repeating animation stops after one pass: the app turns the square from the frame callback and paints it on a canvas",
    ),
];
const TAURI: &[(&str, &str)] = &[(
    "renderer",
    "the app turns the DMA-BUF renderer of WebKitGTK off, as Tauri apps do on Wayland with NVIDIA",
)];
const DIOXUS: &[(&str, &str)] = &[(
    "list",
    "no virtual list: the app renders the rows of the viewport from the scroll offset",
)];

/// Every known framework and edition.
pub const ALL: &[Framework] = &[
    Framework {
        name: "zgui",
        app: "zgui",
        package: "zgui",
        build: "",
        marks: true,
        default: true,
        flags: NO_FLAGS,
    },
    Framework {
        name: "gpui-kit",
        app: "gpui-kit",
        package: "gpui-kit",
        build: "",
        marks: false,
        default: true,
        flags: GPUI,
    },
    Framework {
        name: "gpui-kit-tuned",
        app: "gpui-kit",
        package: "gpui-kit",
        build: "tuned",
        marks: false,
        default: true,
        flags: GPUI,
    },
    Framework {
        name: "gpui-kit-fast",
        app: "gpui-kit-fast",
        package: "gpui-fast",
        build: "",
        marks: false,
        default: false,
        flags: GPUI,
    },
    Framework {
        name: "freya",
        app: "freya",
        package: "freya",
        build: "",
        marks: false,
        default: true,
        flags: FREYA,
    },
    Framework {
        name: "freya-056a5b4",
        app: "freya-056a5b4",
        package: "freya",
        build: "",
        marks: false,
        default: false,
        flags: FREYA,
    },
    Framework {
        name: "vizia",
        app: "vizia",
        package: "vizia",
        build: "",
        marks: false,
        default: true,
        flags: VIZIA,
    },
    Framework {
        name: "floem",
        app: "floem",
        package: "floem",
        build: "",
        marks: false,
        default: true,
        flags: FLOEM,
    },
    Framework {
        name: "iced",
        app: "iced",
        package: "iced",
        build: "",
        marks: false,
        default: true,
        flags: ICED,
    },
    Framework {
        name: "iced-tuned",
        app: "iced",
        package: "iced",
        build: "tuned",
        marks: false,
        default: true,
        flags: ICED,
    },
    Framework {
        name: "egui",
        app: "egui",
        package: "eframe",
        build: "",
        marks: false,
        default: true,
        flags: EGUI,
    },
    Framework {
        name: "dioxus",
        app: "dioxus",
        package: "dioxus-native",
        build: "",
        marks: false,
        default: true,
        flags: DIOXUS,
    },
    Framework {
        name: "dioxus-skia",
        app: "dioxus",
        package: "dioxus-native",
        build: "skia",
        marks: false,
        default: false,
        flags: DIOXUS,
    },
    Framework {
        name: "tauri",
        app: "tauri",
        package: "tauri",
        build: "",
        marks: false,
        default: true,
        flags: TAURI,
    },
];

impl Framework {
    /// Returns the names of the frameworks that run by default.
    pub fn defaults() -> Vec<String> {
        ALL.iter()
            .filter(|f| f.default)
            .map(|f| f.name.to_owned())
            .collect()
    }

    /// Returns the framework with a name.
    pub fn by_name(name: &str) -> Option<&'static Framework> {
        ALL.iter().find(|f| f.name == name)
    }

    /// Returns the version of the framework crate that the app builds with, from the app's
    /// `Cargo.lock`: `(version, source)`. The source is `crates.io`, a git URL with its short
    /// revision, or a local checkout for a crate that a path patch replaces.
    pub fn version(&self, root: &Path) -> Option<(String, String)> {
        let lock = std::fs::read_to_string(root.join("apps").join(self.app).join("Cargo.lock"));
        let lock = lock.ok()?;
        let name = format!("name = \"{}\"", self.package);
        let entry = lock
            .split("[[package]]")
            .find(|entry| entry.lines().any(|line| line == name))?;
        let field = |key: &str| {
            entry
                .lines()
                .find_map(|line| line.strip_prefix(key)?.strip_prefix(" = \""))
                .map(|value| value.trim_end_matches('"').to_owned())
        };
        let source = match field("source") {
            None => "local checkout".to_owned(),
            Some(source) if source.starts_with("registry+") => "crates.io".to_owned(),
            Some(source) => {
                let source = source.trim_start_matches("git+");
                let (url, rev) = source.split_once('#').unwrap_or((source, ""));
                let url = url.split('?').next().unwrap_or(url);
                format!("git {url} @ {}", rev.get(..7).unwrap_or(rev))
            }
        };
        Some((field("version")?, source))
    }

    /// Returns the binary name of the app, `<app>-bench`.
    pub fn binary(&self) -> String {
        format!("{}-bench", self.app)
    }

    /// Returns the release binary of the app's build.
    pub fn binary_path(&self, root: &Path) -> PathBuf {
        root.join("apps")
            .join(self.app)
            .join("target")
            .join(self.build)
            .join("release")
            .join(self.binary())
    }

    /// Returns the binary of the app's size build, which `just build-min` makes: the same build
    /// optimized for the smallest file. No benchmark runs it.
    pub fn min_binary_path(&self, root: &Path) -> PathBuf {
        let dir = if self.build.is_empty() {
            "min".to_owned()
        } else {
            format!("min-{}", self.build)
        };
        root.join("apps")
            .join(self.app)
            .join("target")
            .join(dir)
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
