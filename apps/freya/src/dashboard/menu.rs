//! The export button and freya's `Menu`, which it opens under itself.

use freya::prelude::*;

use super::{color, primary_button, raised, spec, text};

/// The export button with its menu. It holds whether the menu is open: a press on the button
/// flips it, and the menu closes itself on a press anywhere.
#[derive(PartialEq)]
pub struct ExportMenu;

impl Component for ExportMenu {
    fn render(&self) -> impl IntoElement {
        let mut open = use_state(|| false);
        let button = primary_button(spec::EXPORT_LABEL, spec::EXPORT_WIDTH)
            .on_press(move |_| open.toggle());

        rect().child(raised(button)).maybe_child(open().then(|| {
            // The menu lies over the content, its left edge under the button's.
            rect()
                .position(
                    Position::new_absolute()
                        .top(spec::MENU_Y - (spec::TOPBAR_HEIGHT - spec::CONTROL_HEIGHT) / 2.0)
                        .left(0.0),
                )
                .child(
                    Menu::new()
                        .theme(
                            MenuContainerThemePartial::new()
                                .background(color(spec::CARD_BG))
                                .padding(Gaps::new_all(spec::MENU_PADDING))
                                .shadow(super::shadow(spec::SHADOW_CARD).color)
                                .border_fill(color(spec::RULE))
                                .corner_radius(8.0),
                        )
                        .on_close(move |_| open.set(false))
                        .children(spec::MENU_ITEMS.map(|name| item(name).into_element())),
                )
        }))
    }
}

/// Returns one item of the menu with its label.
fn item(name: &'static str) -> MenuItem {
    MenuItem::new()
        .theme(
            MenuItemThemePartial::new()
                .hover_background(color(spec::MENU_HOVER))
                .corner_radius(6.0)
                .color(color(spec::TEXT)),
        )
        .padding(0.0)
        .child(
            rect()
                .width(Size::px(spec::MENU_WIDTH - 2.0 * spec::MENU_PADDING))
                .height(Size::px(spec::MENU_ITEM_HEIGHT))
                .padding(Gaps::new(0.0, 12.0, 0.0, 12.0))
                .main_align(Alignment::Center)
                .child(text(name, spec::FONT_SIZE, FontWeight::NORMAL, spec::TEXT)),
        )
}
