//! The filter card: floem's labeled checkboxes for the statuses, its toggle button as the compare
//! switch, and the Reset and Apply buttons. The controls write the signals of the state.

use floem::prelude::*;

use super::State;
use super::style::{Weight, card, color, px, text};
use super::topbar::{primary_button, secondary_button};
use crate::dash::{self, Status};

/// The window y of the card top, which the spec positions count from.
const CARD_TOP: f32 = dash::TABLE_TOP - dash::GAP - dash::ROW2_HEIGHT;

/// The window x of the checkbox labels.
const LABEL_X: f32 = 968.0;

/// Returns the filter card.
pub fn view(state: State) -> impl IntoView {
    // The title at the top of the band above the first checkbox row.
    let title = Container::new(text(
        dash::FILTERS_TITLE,
        dash::TITLE_FONT,
        Weight::Semibold,
        dash::TEXT,
    ))
    .style(|s| {
        s.flex_shrink(0.0_f32)
            .height(px(dash::CHECK_TOP - CARD_TOP - dash::CARD_PADDING))
            .items_start()
    });
    let checks = Stack::vertical_from_iter(
        Status::ALL
            .iter()
            .enumerate()
            .map(|(k, status)| check(status.label(), state.shown[k])),
    );
    let compare_label = text(dash::COMPARE_LABEL, dash::FONT_SIZE, Weight::Normal, dash::TEXT);
    let compare = Stack::horizontal((compare_label, switch(state.compare))).style(|s| {
        s.flex_shrink(0.0_f32)
            .width_full()
            .height(28.0)
            .margin_top(px(
                dash::COMPARE_Y - dash::CHECK_TOP - 4.0 * dash::CHECK_STRIDE
            ))
            .items_center()
            .justify_between()
    });
    let spacer = Empty::new().style(|s| s.flex_grow(1.0_f32));
    let buttons = Stack::horizontal((
        secondary_button(dash::RESET_LABEL, dash::BUTTON_WIDTH, move || state.reset()),
        primary_button(dash::APPLY_LABEL, dash::BUTTON_WIDTH, || {}),
    ))
    .style(|s| {
        s.flex_shrink(0.0_f32)
            .width_full()
            .justify_end()
            .col_gap(px(dash::APPLY_X - dash::RESET_X - dash::BUTTON_WIDTH))
    });
    Stack::vertical((title, checks, compare, spacer, buttons)).style(|s| {
        card(s)
            .width(px(dash::FILTERS_WIDTH))
            .height(px(dash::ROW2_HEIGHT))
            .padding(px(dash::CARD_PADDING))
    })
}

/// A status row: floem's labeled checkbox, its box 18 px with the label 10 px beside it.
fn check(label: &'static str, shown: RwSignal<bool>) -> impl IntoView {
    Checkbox::labeled_rw(shown, move || label).style(move |s| {
        let checked = shown.get();
        s.height(px(dash::CHECK_STRIDE))
            .padding(0.0)
            .col_gap(px(LABEL_X - dash::CHECK_X - dash::CHECK_SIZE))
            .color(color(dash::TEXT))
            .background(Color::TRANSPARENT)
            .hover(|s| s.background(Color::TRANSPARENT))
            .active(|s| s.background(Color::TRANSPARENT))
            .class(CheckboxClass, move |s| {
                let (fill, mark) = if checked {
                    (dash::PRIMARY, 0xffffff)
                } else {
                    (dash::CARD_BG, dash::CARD_BG)
                };
                s.size(px(dash::CHECK_SIZE), px(dash::CHECK_SIZE))
                    .padding(2.0)
                    .border(1.0)
                    .border_radius(4.0)
                    .border_color(color(if checked { dash::PRIMARY } else { dash::INPUT_BORDER }))
                    .background(color(fill))
                    .color(color(mark))
                    .hover(|s| s.background(color(fill)))
                    .active(|s| s.background(color(fill)))
            })
    })
}

/// The compare switch: floem's toggle button as a 40x22 track with an 18 px white knob.
fn switch(compare: RwSignal<bool>) -> impl IntoView {
    ToggleButton::new_rw(compare)
        .toggle_style(move |s| {
            let track = if compare.get() {
                dash::PRIMARY
            } else {
                dash::SWITCH_OFF
            };
            s.handle_color(color(0xffffff))
                .accent_color(color(track))
                .handle_inset(2.0)
                .circle_rad(9.0)
        })
        .style(move |s| {
            let track = color(if compare.get() {
                dash::PRIMARY
            } else {
                dash::SWITCH_OFF
            });
            s.flex_shrink(0.0_f32)
                .size(px(dash::SWITCH_WIDTH), px(dash::SWITCH_HEIGHT))
                .padding(0.0)
                .border(0.0)
                .border_radius(px(dash::SWITCH_HEIGHT / 2.0))
                .hover(|s| s.background(track))
                .active(|s| s.background(track))
        })
}
