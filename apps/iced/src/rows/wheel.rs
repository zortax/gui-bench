//! A wrapper that turns wheel line steps into pixel distances before its content sees them. The
//! scrollable moves 60 px per line and passes pixel distances through unchanged, so one wheel
//! detent moves the content by `DETENT_PIXELS`.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{Clipboard, Shell, Widget, overlay, renderer};
use iced::mouse::{self, ScrollDelta};
use iced::{Element, Event, Length, Rectangle, Size, Vector};

/// The axis distance of one wheel detent in logical pixels.
const DETENT_PIXELS: f32 = 15.0;

/// Wraps an element and rewrites its wheel line steps as pixel distances.
pub struct Wheel<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> Wheel<'a, Message, Theme, Renderer> {
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

/// Returns the event with line steps converted to pixels, or `None` when it needs no change.
fn convert(event: &Event) -> Option<Event> {
    match event {
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: ScrollDelta::Lines { x, y },
        }) => Some(Event::Mouse(mouse::Event::WheelScrolled {
            delta: ScrollDelta::Pixels {
                x: x * DETENT_PIXELS,
                y: y * DETENT_PIXELS,
            },
        })),
        _ => None,
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Wheel<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let converted = convert(event);
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            converted.as_ref().unwrap_or(event),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Wheel<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(wheel: Wheel<'a, Message, Theme, Renderer>) -> Self {
        Element::new(wheel)
    }
}
