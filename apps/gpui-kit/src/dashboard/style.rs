//! The look of the dashboard: the gpui-component theme set to the colors of the spec, so the
//! kit's input, checkboxes, switches, buttons, toggles and menu draw in them, and the card and
//! shadow styles of the dashboard's own containers.

use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Theme, h_flex};
use gpui_kit::*;

use super::spec;

/// The primary color under the pointer and while pressed, one step darker than `PRIMARY`.
const PRIMARY_HOVER: u32 = 0x4338ca;

/// Returns a spec color.
pub fn color(value: u32) -> Hsla {
    rgb(value).into()
}

/// Returns a spec shadow as a GPUI box shadow.
pub fn shadow(shadow: spec::Shadow) -> Vec<BoxShadow> {
    let mut color = color(shadow.color);
    color.a = shadow.alpha;
    vec![BoxShadow::new(px(shadow.x), px(shadow.y), color).blur_radius(px(shadow.blur))]
}

/// Returns a card: white, rounded and with the card shadow.
pub fn card() -> Div {
    div()
        .bg(rgb(spec::CARD_BG))
        .rounded(px(spec::CARD_RADIUS))
        .shadow(shadow(spec::SHADOW_CARD))
}

/// Returns the switch box of the spec with `switch` in it. The kit's switch comes in fixed
/// sizes; the default one is centered on the box.
pub fn switch_box(switch: Switch) -> Div {
    h_flex()
        .flex_none()
        .w(px(spec::SWITCH_WIDTH))
        .h(px(spec::SWITCH_HEIGHT))
        .justify_center()
        .child(switch)
}

/// Returns `view` as a child element. The view's root element lays itself out; `style` is that
/// same layout, which only the `tuned` edition reads.
#[cfg(not(feature = "tuned"))]
pub fn embed<V: Render>(view: &Entity<V>, _style: StyleRefinement) -> AnyElement {
    view.clone().into_any_element()
}

/// Returns `view` as a cached child element laid out at `style`: GPUI reuses what the view drew
/// last until the view is notified.
#[cfg(feature = "tuned")]
pub fn embed<V: Render>(view: &Entity<V>, style: StyleRefinement) -> AnyElement {
    view.clone().cached(style).into_any_element()
}

/// Sets the theme colors and radius that the kit's components read to the spec.
pub fn apply_theme(cx: &mut App) {
    Theme::update(cx, |theme| {
        theme.radius = px(8.0);
        theme.foreground = color(spec::TEXT);
        theme.muted_foreground = color(spec::MUTED);
        theme.border = color(spec::RULE);
        theme.input = color(spec::INPUT_BORDER);
        theme.ring = color(spec::PRIMARY);
        theme.primary = color(spec::PRIMARY);
        theme.primary_hover = color(PRIMARY_HOVER);
        theme.primary_active = color(PRIMARY_HOVER);
        theme.primary_foreground = color(spec::PRIMARY_TEXT);
        theme.button_primary = color(spec::PRIMARY);
        theme.button_primary_hover = color(PRIMARY_HOVER);
        theme.button_primary_active = color(PRIMARY_HOVER);
        theme.button_primary_foreground = color(spec::PRIMARY_TEXT);
        theme.button = color(spec::SECONDARY_BG);
        theme.button_hover = color(spec::INPUT_BG);
        theme.button_active = color(spec::RULE);
        theme.button_foreground = color(spec::TEXT);
        theme.switch = color(spec::SWITCH_OFF);
        theme.switch_thumb = color(spec::CARD_BG);
        theme.popover = color(spec::CARD_BG);
        theme.popover_foreground = color(spec::TEXT);
        theme.accent = color(spec::MENU_HOVER);
        theme.accent_foreground = color(spec::TEXT);
    });
}
