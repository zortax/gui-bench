//! The top bar: the page title, gpui-component's text input for the customer search, the
//! export button (the kit's primary button) and the user avatar.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::h_flex;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::*;

use super::spec;
use super::style::{color, shadow};

/// The space between the search input and the export button.
const SEARCH_GAP: f32 = spec::EXPORT_X - spec::SEARCH_X - spec::SEARCH_WIDTH;
/// The space between the export button and the avatar.
const EXPORT_GAP: f32 = spec::TOPBAR_AVATAR_X - spec::EXPORT_X - spec::EXPORT_WIDTH;

/// Returns the top bar with the search input of `search`.
pub fn topbar(search: &Entity<InputState>) -> Div {
    h_flex()
        .flex_none()
        .w_full()
        .h(px(spec::TOPBAR_HEIGHT))
        // One pixel on top to match the bottom border, so the row centers on y 32.
        .pt(px(1.0))
        .px(px(spec::TOPBAR_TITLE_X - spec::SIDEBAR_WIDTH))
        .border_b_1()
        .border_color(rgb(spec::RULE))
        .bg(rgb(spec::TOPBAR_BG))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(spec::TITLE_FONT + 4.0))
                .font_weight(FontWeight::SEMIBOLD)
                .child(spec::PAGE_TITLE),
        )
        .child(
            Input::new(search)
                .flex_none()
                .w(px(spec::SEARCH_WIDTH))
                .h_9()
                .bg(rgb(spec::INPUT_BG))
                .border_color(rgb(spec::INPUT_BORDER)),
        )
        .child(
            Button::new("export")
                .primary()
                .label(spec::EXPORT_LABEL)
                .flex_none()
                .ml(px(SEARCH_GAP))
                .w(px(spec::EXPORT_WIDTH))
                .h(px(spec::CONTROL_HEIGHT))
                .font_weight(FontWeight::SEMIBOLD)
                .shadow(shadow(spec::SHADOW_SMALL)),
        )
        .child(
            h_flex()
                .flex_none()
                .ml(px(EXPORT_GAP))
                .size(px(spec::CONTROL_HEIGHT))
                .justify_center()
                .rounded_full()
                .bg(rgb(spec::USER_COLOR))
                .text_size(px(13.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(color(spec::PRIMARY_TEXT))
                .child(spec::USER_INITIALS),
        )
}
