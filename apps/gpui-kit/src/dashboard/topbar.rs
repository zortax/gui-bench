//! The top bar: its own view with the title of the page, gpui-component's text input for the
//! customer search, its switch for the live updates with the spinner beside it, the export
//! button (the kit's primary button) with its dropdown menu, and the user avatar.

use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::h_flex;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::component::switch::Switch;
use gpui_kit::*;

use super::live::Live;
use super::spec;
use super::style::{color, shadow, switch_box};

/// The space between the search input and the live switch.
const SEARCH_GAP: f32 = spec::LIVE_X - spec::SEARCH_X - spec::SEARCH_WIDTH;
/// The space between the live switch and the spinner.
const LIVE_GAP: f32 = spec::SPINNER_X - spec::LIVE_X - spec::SWITCH_WIDTH;
/// The space between the spinner and the export button.
const SPINNER_GAP: f32 = spec::EXPORT_X - spec::SPINNER_X - spec::SPINNER_SIZE;
/// The space between the export button and the avatar.
const EXPORT_GAP: f32 = spec::TOPBAR_AVATAR_X - spec::EXPORT_X - spec::EXPORT_WIDTH;
/// The label of a menu item from the left of the item, past the item's own padding of 8 px.
const MENU_LABEL_INSET: f32 = 12.0 - 8.0;

/// A rounded square that fills its box; `text_color` paints it. GPUI rotates only `svg()`
/// elements (`Transformation`), so the spinner is an inline SVG.
const SQUARE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect width="24" height="24" rx="6" ry="6"/></svg>"#;

/// The top bar.
pub struct TopBar {
    /// The active nav item, whose name is the title.
    active: usize,
    search: Entity<InputState>,
    live: Entity<Live>,
}

impl TopBar {
    pub fn new(search: Entity<InputState>, live: Entity<Live>) -> Self {
        Self {
            active: 0,
            search,
            live,
        }
    }

    /// Shows the name of nav item `active` as the title.
    pub fn set_active(&mut self, active: usize, cx: &mut Context<Self>) {
        self.active = active;
        cx.notify();
    }

    /// Turns the live updates on or off.
    fn set_live(&mut self, on: bool, cx: &mut Context<Self>) {
        self.live.update(cx, |live, cx| live.set(on, cx));
        cx.notify();
    }
}

/// Returns the spinner, which turns once per period by a repeating linear animation, the way
/// gpui-component's own `Spinner` turns its icon.
fn spinner() -> impl IntoElement {
    svg()
        .data(SQUARE)
        .size_full()
        .text_color(rgb(spec::PRIMARY))
        .with_animation(
            "spin",
            Animation::new(Duration::from_millis(spec::SPINNER_PERIOD_MS)).repeat(),
            |square, delta| square.with_transformation(Transformation::rotate(percentage(delta))),
        )
}

/// Returns the export button, which opens the kit's dropdown menu under it.
fn export() -> impl IntoElement {
    Button::new("export")
        .primary()
        .label(spec::EXPORT_LABEL)
        .w(px(spec::EXPORT_WIDTH))
        .h(px(spec::CONTROL_HEIGHT))
        .font_weight(FontWeight::SEMIBOLD)
        .shadow(shadow(spec::SHADOW_SMALL))
        .dropdown_menu(|menu, _, _| {
            let menu = menu
                .min_w(px(spec::MENU_WIDTH))
                .max_w(px(spec::MENU_WIDTH));
            spec::MENU_ITEMS.iter().fold(menu, |menu, label| {
                menu.item(PopupMenuItem::element(|_, _| {
                    h_flex()
                        .h(px(spec::MENU_ITEM_HEIGHT))
                        .pl(px(MENU_LABEL_INSET))
                        .text_size(px(spec::FONT_SIZE))
                        .child(*label)
                }))
            })
        })
}

impl Render for TopBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let live = self.live.read(cx).on;
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
                    .child(spec::NAV_ITEMS[self.active]),
            )
            .child(
                Input::new(&self.search)
                    .flex_none()
                    .w(px(spec::SEARCH_WIDTH))
                    .h_9()
                    .bg(rgb(spec::INPUT_BG))
                    .border_color(rgb(spec::INPUT_BORDER)),
            )
            .child(
                switch_box(
                    Switch::new("live")
                        .checked(live)
                        .accessibility_label(spec::LIVE_LABEL)
                        .on_click(cx.listener(|this, on: &bool, _, cx| this.set_live(*on, cx))),
                )
                .ml(px(SEARCH_GAP)),
            )
            .child(
                div()
                    .flex_none()
                    .ml(px(LIVE_GAP))
                    .size(px(spec::SPINNER_SIZE))
                    .children(live.then(spinner)),
            )
            .child(div().flex_none().ml(px(SPINNER_GAP)).child(export()))
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
}
