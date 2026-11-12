//! The virtual scenario: the rows of the rows scenario, but only the visible ones are built. iced
//! has no virtualized list; its documented way is to track the viewport with
//! `Scrollable::on_scroll` and build the visible range, with fixed-height spaces standing in for
//! the rows above and below it. The rows and the scrollbar are those of the rows scenario.

use iced::widget::scrollable::{Direction, Viewport};
use iced::widget::{Column, scrollable, space};
use iced::{Element, Length, Theme};

use crate::page;
use crate::rows::row;
use crate::rows::scrollbar;
use crate::spec;

/// The rows built past each edge of the viewport.
const OVERSCAN: usize = 4;

/// The list state: the row count and the visible part of the content.
struct VirtualRows {
    count: usize,
    offset: f32,
    height: f32,
}

#[derive(Debug, Clone)]
enum Message {
    /// The list scrolled or changed size.
    Scrolled(Viewport),
    /// A row was pressed; the rows have no action.
    Row(row::Message),
}

/// Opens the window and runs the event loop until the window closes.
pub fn run(count: usize, title: &'static str, app_id: &str) -> iced::Result {
    let boot = move || VirtualRows {
        count,
        offset: 0.0,
        height: spec::WINDOW.1,
    };
    iced::application(boot, VirtualRows::update, VirtualRows::view)
        .title(title)
        .theme(Theme::Light)
        .style(page::style)
        .window(page::window(app_id))
        .run()
}

impl VirtualRows {
    fn update(&mut self, message: Message) {
        match message {
            Message::Scrolled(viewport) => {
                self.offset = viewport.absolute_offset().y;
                self.height = viewport.bounds().height;
            }
            Message::Row(_) => {}
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let first = ((self.offset / spec::ROW_HEIGHT) as usize)
            .saturating_sub(OVERSCAN)
            .min(self.count);
        let last = (((self.offset + self.height) / spec::ROW_HEIGHT).ceil() as usize + OVERSCAN)
            .min(self.count);
        let above = space().height(first as f32 * spec::ROW_HEIGHT);
        let below = space().height((self.count - last) as f32 * spec::ROW_HEIGHT);
        let rows: Element<'_, row::Message> = Column::new()
            .push(above)
            .extend((first..last).map(row::row))
            .push(below)
            .width(Length::Fill)
            .into();
        scrollable(rows.map(Message::Row))
            .direction(Direction::Vertical(scrollbar()))
            .width(Length::Fill)
            .height(Length::Fill)
            .on_scroll(Message::Scrolled)
            .into()
    }
}
