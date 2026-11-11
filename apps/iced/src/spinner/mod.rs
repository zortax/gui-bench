//! The spinner scenario: a header whose square turns once a second above static rows. iced has no
//! keyframes for widgets; its documented way to animate is an `Animation` in the model, a
//! `window::frames()` subscription that sends a message on every frame while the animation runs,
//! and a `canvas` program that draws the interpolated state (the `arc` and `gallery` examples).
//! The rotation is an `Animation<bool>` that runs from false to true in one period, linear, and
//! repeats forever, interpolated from 0 to one full turn.

use std::f32::consts::TAU;
use std::time::{Duration, Instant};

use iced::animation::Easing;
use iced::mouse;
use iced::widget::canvas::{self, Path};
use iced::widget::{Column, canvas as canvas_widget, column, scrollable};
use iced::{
    Animation, Element, Length, Point, Rectangle, Renderer, Size, Subscription, Theme, Vector,
    window,
};

use crate::page;
use crate::spec::{self, Ids, Row};

/// How far the corners of the turning square reach past its 24 px box: half the diagonal less
/// half the side, rounded up.
const OVERHANG: f32 = 5.0;

/// The page state: the static rows and the rotation.
struct Spinner {
    rows: Vec<Row>,
    rotation: Animation<bool>,
    now: Instant,
}

/// The message of every presented frame.
#[derive(Debug, Clone, Copy)]
enum Message {
    Frame(Instant),
}

/// Opens the window and runs the event loop until the window closes.
pub fn run(count: usize, title: &'static str, app_id: &str) -> iced::Result {
    iced::application(move || Spinner::new(count), Spinner::update, Spinner::view)
        .subscription(Spinner::subscription)
        .title(title)
        .theme(Theme::Light)
        .style(page::style)
        .window(page::window(app_id))
        .run()
}

impl Spinner {
    fn new(count: usize) -> Self {
        let now = Instant::now();
        let rotation = Animation::new(false)
            .duration(Duration::from_millis(spec::SPINNER_PERIOD_MS))
            .easing(Easing::Linear)
            .repeat_forever()
            .go(true, now);
        Self {
            rows: Ids::default().rows(count),
            rotation,
            now,
        }
    }

    fn update(&mut self, message: Message) {
        let Message::Frame(now) = message;
        self.now = now;
    }

    fn view(&self) -> Element<'_, Message> {
        let angle = self.rotation.interpolate(0.0, TAU, self.now);
        let square = canvas_widget(Square { angle })
            .width(spec::SPINNER_SIZE)
            .height(spec::SPINNER_SIZE);
        let rows = Column::with_children(self.rows.iter().map(page::plain_row)).width(Length::Fill);
        column![
            page::header(spec::status(self.rows.len()), square),
            page::list(scrollable(rows)),
        ]
        .into()
    }

    /// Asks for every frame while the rotation runs, which is forever.
    fn subscription(&self) -> Subscription<Message> {
        if self.rotation.is_animating(self.now) {
            window::frames().map(Message::Frame)
        } else {
            Subscription::none()
        }
    }
}

/// The canvas program of the spinner: a rounded square turned by `angle` around its center.
struct Square {
    angle: f32,
}

impl canvas::Program<Message> for Square {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &Renderer,
        _: &Theme,
        _: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // The frame reaches past the canvas box so the turned corners are not clipped.
        let side = spec::SPINNER_SIZE;
        let clip = Rectangle::new(
            Point::new(-OVERHANG, -OVERHANG),
            Size::new(side + 2.0 * OVERHANG, side + 2.0 * OVERHANG),
        );
        let mut frame = canvas::Frame::with_bounds(renderer, clip);
        frame.translate(Vector::new(side / 2.0, side / 2.0));
        frame.rotate(self.angle);
        let square = Path::rounded_rectangle(
            Point::new(-side / 2.0, -side / 2.0),
            Size::new(side, side),
            spec::SPINNER_RADIUS.into(),
        );
        frame.fill(&square, page::color(spec::SPINNER_COLOR));
        vec![frame.into_geometry()]
    }
}
