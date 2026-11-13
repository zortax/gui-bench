//! The top bar: the title of the page, floem's text input for the search, the live switch with
//! its spinner, the export button with its menu and the avatar, on white with a 1 px bottom
//! border. The search writes the search signal, which filters the orders.

use floem::ViewId;
use floem::action::{add_overlay, remove_overlay};
use floem::kurbo::Point;
use floem::prelude::*;

use super::State;
use super::filters::switch;
use super::live::spinner;
use super::style::{self, Weight, circle, color, px, text};
use crate::dash;

/// The space between the export button and its menu.
const MENU_GAP: f32 = dash::MENU_Y - (dash::TOPBAR_HEIGHT + dash::CONTROL_HEIGHT) / 2.0;

/// Returns the top bar.
pub fn view(state: State) -> impl IntoView {
    let title = Label::derived(move || dash::NAV_ITEMS[state.nav.get()]).style(|s| {
        style::font(s, dash::TITLE_FONT + 4.0, Weight::Semibold, dash::TEXT).flex_shrink(0.0_f32)
    });
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
    let live = switch(state.live.on)
        .style(|s| s.margin_left(px(dash::LIVE_X - dash::SEARCH_X - dash::SEARCH_WIDTH)));
    let spinner = spinner(state.live)
        .style(|s| s.margin_left(px(dash::SPINNER_X - dash::LIVE_X - dash::SWITCH_WIDTH)));
    let export = export()
        .style(|s| s.margin_left(px(dash::EXPORT_X - dash::SPINNER_X - dash::SPINNER_SIZE)));
    let avatar = Container::new(text(dash::USER_INITIALS, 13.0, Weight::Semibold, 0xffffff))
        .style(|s| {
            circle(s, dash::CONTROL_HEIGHT, dash::USER_COLOR)
                .margin_left(px(dash::TOPBAR_AVATAR_X - dash::EXPORT_X - dash::EXPORT_WIDTH))
        });
    let fill = Empty::new().style(|s| s.flex_grow(1.0_f32));
    Stack::horizontal((title, fill, search, live, spinner, export, avatar)).style(|s| {
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

/// The export button and its menu: a floem window overlay under the button, which lies over the
/// content and exists while the menu is open (as floem's dropdown and tooltip place theirs). A
/// click on the button opens and closes it; a click on an item closes it.
fn export() -> impl IntoView {
    let overlay = RwSignal::new(None::<ViewId>);
    let close = move || {
        if let Some(menu) = overlay.get_untracked() {
            remove_overlay(menu);
            overlay.set(None);
        }
    };
    let button = primary_button(dash::EXPORT_LABEL, dash::EXPORT_WIDTH);
    let id = button.id();
    button.action(move || {
        if overlay.get_untracked().is_some() {
            return close();
        }
        let anchor = id.get_visual_rect();
        let at = Point::new(anchor.x0, anchor.y1 + px(MENU_GAP));
        let menu = add_overlay(menu(at, close));
        menu.set_style_parent(id);
        overlay.set(Some(menu));
    })
}

/// The box of the export menu with its items, its top left corner at the window point `at`.
fn menu(at: Point, close: impl Fn() + Copy + 'static) -> impl View {
    let items = dash::MENU_ITEMS.iter().map(|label| menu_item(label, close));
    Stack::vertical_from_iter(items).style(move |s| {
        let s = s
            .inset_left(at.x)
            .inset_top(at.y)
            .width(px(dash::MENU_WIDTH))
            .padding(px(dash::MENU_PADDING))
            .border(1.0)
            .border_radius(8.0)
            .border_color(color(dash::RULE))
            .background(color(dash::CARD_BG));
        style::shadow(s, dash::SHADOW_CARD)
    })
}

/// One item of the export menu, which takes the hover color under the pointer.
fn menu_item(label: &'static str, close: impl Fn() + 'static) -> impl IntoView {
    Container::new(text(label, dash::FONT_SIZE, Weight::Normal, dash::TEXT))
        .style(|s| {
            s.width_full()
                .height(px(dash::MENU_ITEM_HEIGHT))
                .padding_horiz(12.0)
                .items_center()
                .border_radius(6.0)
                .hover(|s| s.background(color(dash::MENU_HOVER)))
        })
        .action(close)
}

/// Returns a primary button: floem's button in the primary color with the small shadow.
pub fn primary_button(label: &'static str, width: f32) -> Button {
    Button::new(text(label, dash::FONT_SIZE, Weight::Semibold, dash::PRIMARY_TEXT))
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
pub fn secondary_button(label: &'static str, width: f32) -> Button {
    Button::new(text(label, dash::FONT_SIZE, Weight::Semibold, dash::TEXT))
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
