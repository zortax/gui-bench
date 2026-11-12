//! The filter card: gpui-component's checkboxes for the statuses, its switch for compare, and
//! its default and primary buttons for Reset and Apply. The controls show the state of the
//! dashboard and call back into it.

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Sizable, h_flex, v_flex};
use gpui_kit::*;

use super::Dashboard;
use super::spec::{self, Status};
use super::style::{card, shadow};

/// The height of the title line, down to the first checkbox row.
const TITLE_HEIGHT: f32 =
    spec::CHECK_TOP - (spec::TABLE_TOP - spec::GAP - spec::ROW2_HEIGHT) - spec::CARD_PADDING;
/// The space between the last checkbox row and the compare row.
const COMPARE_GAP: f32 = spec::COMPARE_Y - spec::CHECK_TOP - 4.0 * spec::CHECK_STRIDE;
/// The height of the compare row.
const COMPARE_HEIGHT: f32 = 28.0;
/// The space between the two buttons.
const BUTTON_GAP: f32 = spec::APPLY_X - spec::RESET_X - spec::BUTTON_WIDTH;

/// Returns the filter card for the state of `dashboard`.
pub fn filters(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> Div {
    let checkboxes = Status::ALL.iter().enumerate().map(|(k, status)| {
        h_flex().flex_none().h(px(spec::CHECK_STRIDE)).child(
            Checkbox::new(("status", k))
                .large()
                .label(status.label())
                .checked(dashboard.shown[k])
                .gap(px(10.0))
                .text_size(px(spec::FONT_SIZE))
                .on_click(cx.listener(move |this, checked: &bool, _, cx| {
                    this.show_status(k, *checked, cx)
                })),
        )
    });
    let compare = h_flex()
        .flex_none()
        .mt(px(COMPARE_GAP))
        .h(px(COMPARE_HEIGHT))
        .justify_between()
        .child(spec::COMPARE_LABEL)
        .child(
            // The kit's switch comes in fixed sizes; the default one is centered on the
            // switch box of the spec.
            h_flex()
                .flex_none()
                .w(px(spec::SWITCH_WIDTH))
                .h(px(spec::SWITCH_HEIGHT))
                .justify_center()
                .child(
                    Switch::new("compare")
                        .checked(dashboard.compare)
                        .on_click(cx.listener(|this, on: &bool, _, cx| this.set_compare(*on, cx))),
                ),
        );
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
