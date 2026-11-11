//! The spinner scenario: a header whose square turns once a second through a CSS keyframe
//! animation (`spin` in `page.css`), above static rows. Blitz runs the animation in its style
//! engine and paints a frame for each animation tick, without rendering a component.
//!
//! Blitz starts that frame loop only from a redraw. With vello-hybrid, whose renderer gets ready
//! after the window opens, blitz-shell paints the first frame in `View::complete_resume`, which,
//! unlike `View::redraw`, asks for no next frame while the document animates, and the redraw that
//! dioxus-native requested before the renderer was ready was dropped. So on this static page the
//! square stands still at its first frame until an input event (such as the pointer entering a
//! row) triggers a redraw; from then on it turns. With Skia it turns from the start.

use dioxus::prelude::*;

use crate::page::{self, Header, PlainRow};
use crate::spec::{self, Ids};

/// The page with `count` static rows and the spinner.
#[component]
pub fn Spinner(count: usize) -> Element {
    let rows = use_hook(|| Ids::default().rows(count));
    rsx! {
        style { {page::CSS} }
        div { class: "page",
            Header { status: spec::status(count),
                div { class: "spinner" }
            }
            div { class: "list",
                for row in rows {
                    PlainRow { key: "{row.id}", row }
                }
            }
        }
    }
}
