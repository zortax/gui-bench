//! The live updates: while the switch of the top bar is on, a timer advances the tick that the
//! metric cards show, and the spinner turns. iced has no keyframes for widgets; its documented
//! way to animate is an `Animation` in the state, a `window::frames()` subscription that sends a
//! message on every frame while the animation runs, and a `canvas` program that draws the
//! interpolated state (the `arc` and `gallery` examples). The timer is `time::every`.

use std::f32::consts::TAU;
use std::time::{Duration, Instant};

use iced::animation::Easing;
use iced::mouse;
use iced::widget::canvas::{self, Path};
use iced::{
    Animation, Element, Point, Rectangle, Renderer, Size, Subscription, Theme, Vector, time, window,
};

use super::style::color;
use super::{Message, spec};

/// How far the corners of the turning square reach past its 24 px box: half the diagonal less
/// half the side, rounded up.
const OVERHANG: f32 = 5.0;

/// The live state.
pub struct Live {
    /// Whether the live switch is on.
    pub on: bool,
    /// The tick the metric cards show; 0 while live is off.
    pub tick: u64,
    /// The turn of the spinner: from false to true in one period, linear, forever.
    rotation: Animation<bool>,
    /// The time of the last frame.
    now: Instant,
}

impl Live {
    /// Live off at tick 0.
    pub fn new() -> Self {
        Self {
            on: false,
            tick: 0,
            rotation: Animation::new(false),
            now: Instant::now(),
        }
    }

    /// Turns the live updates on or off. Turning them on starts the spinner; either way the
    /// metric cards go back to their startup values.
    pub fn set(&mut self, on: bool) {
        self.on = on;
        self.tick = 0;
        if on {
            self.now = Instant::now();
            self.rotation = Animation::new(false)
                .duration(Duration::from_millis(spec::SPINNER_PERIOD_MS))
                .easing(Easing::Linear)
                .repeat_forever()
                .go(true, self.now);
        }
    }

    /// Takes the time of a presented frame.
    pub fn frame(&mut self, now: Instant) {
        self.now = now;
    }

    /// While live is on: the timer of the metric values, and every frame for the spinner.
    pub fn subscription(&self) -> Subscription<Message> {
        if !self.on {
            return Subscription::none();
        }
        Subscription::batch([
            time::every(Duration::from_millis(spec::LIVE_INTERVAL_MS)).map(|_| Message::Tick),
            window::frames().map(Message::Frame),
        ])
    }

    /// Returns the spinner at its current angle.
    pub fn spinner<'a>(&self) -> Element<'a, Message> {
        let angle = self.rotation.interpolate(0.0, TAU, self.now);
        iced::widget::canvas(Square { angle })
            .width(spec::SPINNER_SIZE)
            .height(spec::SPINNER_SIZE)
            .into()
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
        frame.fill(&square, color(spec::PRIMARY));
        vec![frame.into_geometry()]
    }
}
