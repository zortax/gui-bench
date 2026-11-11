//! Stroke icons from the lucide set (ISC license), drawn in the current text color.

use leptos::prelude::*;

/// The icons the explorer uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    ChevronDown,
    Close,
    Download,
    ZoomIn,
    ZoomOut,
    Reset,
    Sun,
    Moon,
    Monitor,
    Info,
    Focus,
    Expand,
    Sidebar,
}

impl IconKind {
    fn paths(self) -> &'static str {
        match self {
            Self::ChevronDown => r#"<path d="m6 9 6 6 6-6"/>"#,
            Self::Close => r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#,
            Self::Download => {
                r#"<path d="M12 15V3"/><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="m7 10 5 5 5-5"/>"#
            }
            Self::ZoomIn => {
                r#"<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/><path d="M11 8v6"/><path d="M8 11h6"/>"#
            }
            Self::ZoomOut => {
                r#"<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/><path d="M8 11h6"/>"#
            }
            Self::Reset => {
                r#"<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/>"#
            }
            Self::Sun => {
                r#"<circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/>"#
            }
            Self::Moon => r#"<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>"#,
            Self::Monitor => {
                r#"<rect width="20" height="14" x="2" y="3" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/>"#
            }
            Self::Info => {
                r#"<circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>"#
            }
            Self::Focus => {
                r#"<circle cx="12" cy="12" r="3"/><path d="M3 7V5a2 2 0 0 1 2-2h2"/><path d="M17 3h2a2 2 0 0 1 2 2v2"/><path d="M21 17v2a2 2 0 0 1-2 2h-2"/><path d="M7 21H5a2 2 0 0 1-2-2v-2"/>"#
            }
            Self::Expand => {
                r#"<path d="m15 3 6 6"/><path d="M21 3h-6v6"/><path d="m9 21-6-6"/><path d="M3 21h6v-6"/>"#
            }
            Self::Sidebar => {
                r#"<rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/>"#
            }
        }
    }
}

/// An icon at the current font size.
#[component]
pub fn Icon(kind: IconKind) -> impl IntoView {
    view! {
        <svg
            class="gx-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
            inner_html=kind.paths()
        ></svg>
    }
}
