//! The page of the header scenarios: a header with the status text and a slot on the right, and
//! a scroll container of rows below it. Rows here have one background and no hover color.

mod header;
mod row;

pub use header::header;
pub use row::{plain_row, row};

use iced::widget::scrollable::{Direction, Scrollable};
use iced::{Color, Element, Length, Size, Theme, theme, window};

use crate::rows::scrollbar;
use crate::rows::wheel::Wheel;
use crate::spec;

/// Returns a color of the spec.
pub fn color(hex: u32) -> Color {
    let (r, g, b) = spec::rgb(hex);
    Color::from_rgb8(r, g, b)
}

/// Returns the undecorated 1280x800 window of every scenario.
pub fn window(app_id: &str) -> window::Settings {
    window::Settings {
        size: Size::new(spec::WINDOW.0, spec::WINDOW.1),
        decorations: false,
        platform_specific: window::settings::PlatformSpecific {
            application_id: app_id.to_owned(),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// The light application style: a white background and the label color for text.
pub fn style<State>(_: &State, _: &Theme) -> theme::Style {
    theme::Style {
        background_color: color(spec::BACKGROUND),
        text_color: color(spec::LABEL_COLOR),
    }
}

/// Returns the scroll container below the header, filling the rest of the window. Wheel
/// detents move it as far as in the rows scenario.
pub fn list<'a, Message: 'a>(scroll: Scrollable<'a, Message>) -> Element<'a, Message> {
    Wheel::new(
        scroll
            .direction(Direction::Vertical(scrollbar()))
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .into()
}
