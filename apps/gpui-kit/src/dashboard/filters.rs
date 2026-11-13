//! The filter state and the filter card: gpui-component's checkboxes for the statuses, its
//! switch for compare, and its default and primary buttons for Reset and Apply. The controls
//! show the state and write it.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Sizable, h_flex, v_flex};
use gpui_kit::*;

use super::overview::Overview;
use super::spec::{self, Status};
use super::style::{card, shadow, switch_box};

/// The height of the title line, down to the first checkbox row.
const TITLE_HEIGHT: f32 =
    spec::CHECK_TOP - (spec::TABLE_TOP - spec::GAP - spec::ROW2_HEIGHT) - spec::CARD_PADDING;
/// The space between the last checkbox row and the compare row.
const COMPARE_GAP: f32 = spec::COMPARE_Y - spec::CHECK_TOP - 4.0 * spec::CHECK_STRIDE;
/// The height of the compare row.
const COMPARE_HEIGHT: f32 = 28.0;
/// The space between the two buttons.
const BUTTON_GAP: f32 = spec::APPLY_X - spec::RESET_X - spec::BUTTON_WIDTH;

/// The values the controls write and the chart and the orders read. It outlives the overview
/// page, and the views that show it observe it.
pub struct Filters {
    /// The chart series, an index into `SEGMENTS`.
    pub series: usize,
    /// Whether the chart shows the previous period.
    pub compare: bool,
    /// Whether the orders of `Status::ALL[k]` show.
    pub shown: [bool; 4],
    /// Whether the orders are sorted by amount.
    pub sorted: bool,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            series: 0,
            compare: false,
            shown: [true; 4],
            sorted: false,
        }
    }
}

impl Filters {
    /// Shows or hides the orders of status `k`.
    pub fn show_status(&mut self, k: usize, shown: bool, cx: &mut Context<Self>) {
        self.shown[k] = shown;
        cx.notify();
    }

    /// Turns the previous period of the chart on or off.
    pub fn set_compare(&mut self, compare: bool, cx: &mut Context<Self>) {
        self.compare = compare;
        cx.notify();
    }

    /// Shows the series of the checked segment. A click on the selected segment unchecks it in
    /// the group, and the selection stays.
    pub fn select_series(&mut self, checked: &[bool], cx: &mut Context<Self>) {
        if let Some(series) = checked.iter().position(|checked| *checked) {
            self.series = series;
            cx.notify();
        }
    }

    /// Sorts the orders by amount, or brings the order of the data back.
    pub fn toggle_sorted(&mut self, cx: &mut Context<Self>) {
        self.sorted = !self.sorted;
        cx.notify();
    }

    /// Goes back to the startup state.
    pub fn reset(&mut self, cx: &mut Context<Self>) {
        *self = Self::default();
        cx.notify();
    }
}

/// Returns the filter card for `filters`, in the overview.
pub fn filters(filters: &Entity<Filters>, cx: &mut Context<Overview>) -> Div {
    let state = filters.read(cx);
    let (shown, compare) = (state.shown, state.compare);
    let checkboxes = Status::ALL.iter().enumerate().map(|(k, status)| {
        let filters = filters.clone();
        h_flex().flex_none().h(px(spec::CHECK_STRIDE)).child(
            Checkbox::new(("status", k))
                .large()
                .label(status.label())
                .checked(shown[k])
                .gap(px(10.0))
                .text_size(px(spec::FONT_SIZE))
                .on_click(move |checked: &bool, _, cx| {
                    filters.update(cx, |filters, cx| filters.show_status(k, *checked, cx))
                }),
        )
    });
    let compare = h_flex()
        .flex_none()
        .mt(px(COMPARE_GAP))
        .h(px(COMPARE_HEIGHT))
        .justify_between()
        .child(spec::COMPARE_LABEL)
        .child(switch_box(Switch::new("compare").checked(compare).on_click({
            let filters = filters.clone();
            move |on: &bool, _, cx| filters.update(cx, |filters, cx| filters.set_compare(*on, cx))
        })));
    let buttons = h_flex()
        .flex_none()
        .mt_auto()
        .h(px(spec::CONTROL_HEIGHT))
        .justify_end()
        .gap(px(BUTTON_GAP))
        .child(
            Button::new("reset")
                .label(spec::RESET_LABEL)
                .w(px(spec::BUTTON_WIDTH))
                .h(px(spec::CONTROL_HEIGHT))
                .font_weight(FontWeight::SEMIBOLD)
                .on_click(cx.listener(|this, _, window, cx| this.reset(window, cx))),
        )
        .child(
            Button::new("apply")
                .primary()
                .label(spec::APPLY_LABEL)
                .w(px(spec::BUTTON_WIDTH))
                .h(px(spec::CONTROL_HEIGHT))
                .font_weight(FontWeight::SEMIBOLD)
                .shadow(shadow(spec::SHADOW_SMALL)),
        );
    card()
        .flex_none()
        .w(px(spec::FILTERS_WIDTH))
        .h_full()
        .p(px(spec::CARD_PADDING))
        .child(
            v_flex()
                .size_full()
                .child(
                    div()
                        .flex_none()
                        .h(px(TITLE_HEIGHT))
                        .text_size(px(spec::TITLE_FONT))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(spec::FILTERS_TITLE),
                )
                .children(checkboxes)
                .child(compare)
                .child(buttons),
        )
}
