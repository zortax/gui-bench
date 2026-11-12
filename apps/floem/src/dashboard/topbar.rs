//! The top bar: the page title, floem's text input for the search, the export button and the
//! avatar, on white with a 1 px bottom border. The search writes the search signal, which
//! filters the orders.

use floem::prelude::*;

use super::State;
use super::style::{self, Weight, circle, color, px, text};
use crate::dash;

/// Returns the top bar.
pub fn view(state: State) -> impl IntoView {
    let title = text(
        dash::PAGE_TITLE,
        dash::TITLE_FONT + 4.0,
        Weight::Semibold,
        dash::TEXT,
    )
    .style(|s| s.flex_shrink(0.0_f32));
    let search = TextInput::new(state.search)
        .placeholder(dash::SEARCH_PLACEHOLDER)
        .style(|s| {
            s.flex_shrink(0.0_f32)
                .width(px(dash::SEARCH_WIDTH))
                .height(px(dash::CONTROL_HEIGHT))
                .padding_horiz(12.0)
                .padding_vert(0.0)
                .items_center()
                .border(1.0)
                .border_radius(8.0)
                .border_color(color(dash::INPUT_BORDER))
                .background(color(dash::INPUT_BG))
                .hover(|s| s.background(color(dash::INPUT_BG)))
                .focus(|s| {
                    s.border_color(color(dash::PRIMARY))
                        .hover(|s| s.background(color(dash::INPUT_BG)))
                })
        });
    let export = primary_button(dash::EXPORT_LABEL, dash::EXPORT_WIDTH, || {});
    let avatar = Container::new(text(dash::USER_INITIALS, 13.0, Weight::Semibold, 0xffffff))
        .style(|s| {
            circle(s, dash::CONTROL_HEIGHT, dash::USER_COLOR)
                .margin_left(px(dash::TOPBAR_AVATAR_X - dash::EXPORT_X - dash::EXPORT_WIDTH))
        });
    // The space between the search and the export button gives way first in a narrow window.
    let fill = Empty::new().style(|s| s.flex_grow(1.0_f32));
    let gap = Empty::new().style(|s| {
        s.flex_shrink(1.0_f32)
            .width(px(dash::EXPORT_X - dash::SEARCH_X - dash::SEARCH_WIDTH))
    });
    Stack::horizontal((title, fill, search, gap, export, avatar)).style(|s| {
        s.flex_shrink(0.0_f32)
            .width_full()
            .height(px(dash::TOPBAR_HEIGHT))
            .padding_horiz(px(dash::CONTENT_PADDING))
            .items_center()
            .border_bottom(1.0)
            .border_color(color(dash::RULE))
            .background(color(dash::TOPBAR_BG))
    })
}

/// Returns a primary button: floem's button in the primary color with the small shadow.
pub fn primary_button(label: &'static str, width: f32, action: impl Fn() + 'static) -> Button {
    Button::new(text(label, dash::FONT_SIZE, Weight::Semibold, dash::PRIMARY_TEXT))
        .action(action)
        .style(move |s| {
            let s = s
                .flex_shrink(0.0_f32)
                .width(px(width))
                .height(px(dash::CONTROL_HEIGHT))
                .padding(0.0)
                .border(0.0)
                .border_radius(8.0)
                .background(color(dash::PRIMARY))
                .hover(|s| s.background(color(0x4338ca)))
                .active(|s| s.background(color(0x3730a3)));
            style::shadow(s, dash::SHADOW_SMALL)
        })
}

/// Returns a secondary button: floem's button in white with a 1 px border.
pub fn secondary_button(label: &'static str, width: f32, action: impl Fn() + 'static) -> Button {
    Button::new(text(label, dash::FONT_SIZE, Weight::Semibold, dash::TEXT))
        .action(action)
        .style(move |s| {
            s.flex_shrink(0.0_f32)
                .width(px(width))
                .height(px(dash::CONTROL_HEIGHT))
                .padding(0.0)
                .border(1.0)
                .border_radius(8.0)
                .border_color(color(dash::SECONDARY_BORDER))
                .background(color(dash::SECONDARY_BG))
                .hover(|s| s.background(color(dash::INPUT_BG)))
                .active(|s| s.background(color(dash::RULE)))
        })
}
