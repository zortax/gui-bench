//! The export menu. iced has no menu or popover widget (its `pick_list` is a select whose menu
//! always has the width of its button), so the menu is a card of buttons that the dashboard
//! layers over the page with a `stack`, the way iced's `modal` example layers its dialog.

use iced::alignment::Vertical;
use iced::widget::{button, column, container, opaque, pin, text};
use iced::{Background, Border, Element, Length, border};

use super::style::{self, color};
use super::{Message, spec};

/// The left padding of an item's label.
const LABEL_X: f32 = 12.0;

/// Returns the layer of the open menu: the menu pinned under the export button. The rest of the
/// layer is empty and lets the pointer through to the page.
pub fn view<'a>() -> Element<'a, Message> {
    let items = column(spec::MENU_ITEMS.iter().map(|label| item(label)));
    let menu = container(items)
        .padding(spec::MENU_PADDING)
        .width(spec::MENU_WIDTH)
        .style(|_| container::Style {
            border: Border {
                color: color(spec::RULE),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: style::shadow(spec::SHADOW_CARD),
            ..style::filled(spec::CARD_BG, 8)
        });
    pin(opaque(menu)).x(spec::MENU_X).y(spec::MENU_Y).into()
}

/// Returns one item: a button that takes the hover color under the pointer and closes the menu.
fn item<'a>(label: &'a str) -> Element<'a, Message> {
    button(
        text(label)
            .size(spec::FONT_SIZE)
            .height(Length::Fill)
            .align_y(Vertical::Center),
    )
    .padding([0.0, LABEL_X])
    .width(Length::Fill)
    .height(spec::MENU_ITEM_HEIGHT)
    .on_press(Message::MenuItem)
    .style(|_, status| button::Style {
        background: match status {
            button::Status::Hovered | button::Status::Pressed => {
                Some(Background::Color(color(spec::MENU_HOVER)))
            }
            button::Status::Active | button::Status::Disabled => None,
        },
        text_color: color(spec::TEXT),
        border: border::rounded(6),
        ..button::Style::default()
    })
    .into()
}
