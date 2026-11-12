//! The shared looks of the dashboard: spec colors, shadows, the card surface and text labels in
//! a given size, weight and color, all through floem's `Style` builder.

use floem::prelude::*;
use floem::style::Style;
use floem::text::FontWeight;

use crate::dash::{self, Shadow};
pub use crate::page::color;

/// Returns a length of the spec as a floem length value.
pub fn px(value: f32) -> f64 {
    f64::from(value)
}

/// Adds a spec drop shadow to a style.
pub fn shadow(s: Style, shadow: Shadow) -> Style {
    s.box_shadow_h_offset(px(shadow.x))
        .box_shadow_v_offset(px(shadow.y))
        .box_shadow_blur(px(shadow.blur))
        .box_shadow_color(color(shadow.color).with_alpha(shadow.alpha))
}

/// The surface of every card: white, rounded, with the card shadow.
pub fn card(s: Style) -> Style {
    shadow(
        s.flex_shrink(0.0_f32)
            .background(color(dash::CARD_BG))
            .border_radius(px(dash::CARD_RADIUS)),
        dash::SHADOW_CARD,
    )
}

/// The font weights of the spec.
#[derive(Clone, Copy)]
pub enum Weight {
    Normal,
    Semibold,
    Bold,
}

impl Weight {
    fn font(self) -> FontWeight {
        match self {
            Self::Normal => FontWeight::NORMAL,
            Self::Semibold => FontWeight::SEMI_BOLD,
            Self::Bold => FontWeight::BOLD,
        }
    }
}

/// Returns a static text in `size` px, `weight` and the spec color `hex`.
pub fn text(content: impl std::fmt::Display, size: f32, weight: Weight, hex: u32) -> Label {
    Label::new(content).style(move |s| {
        s.font_size(px(size))
            .font_weight(weight.font())
            .color(color(hex))
    })
}

/// A pill: a rounded box of `height` around its text, `padding_x` on each side.
pub fn pill(s: Style, height: f32, padding_x: f32, background: u32) -> Style {
    s.flex_shrink(0.0_f32)
        .height(px(height))
        .padding_horiz(px(padding_x))
        .items_center()
        .justify_center()
        .border_radius(px(height / 2.0))
        .background(color(background))
}

/// A circle of `size` with centered content, as for the avatars.
pub fn circle(s: Style, size: f32, background: u32) -> Style {
    s.flex_shrink(0.0_f32)
        .size(px(size), px(size))
        .items_center()
        .justify_center()
        .border_radius(px(size / 2.0))
        .background(color(background))
}
