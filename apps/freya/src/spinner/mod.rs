//! The spinner scenario: a header whose square turns once a second through a freya animation
//! that restarts when it ends, above static rows.

use freya::animation::*;
use freya::prelude::*;

use crate::page::{self, PlainRow};
use crate::spec::{self, Ids};

/// Opens the spinner window and runs the event loop until it closes.
pub fn run(title: &'static str, count: usize) {
    page::launch_app(title, SpinnerPage { count });
}

/// The page with `count` static rows and the spinner.
struct SpinnerPage {
    count: usize,
}

impl App for SpinnerPage {
    fn render(&self) -> impl IntoElement {
        let rows = Ids::default()
            .rows(self.count)
            .into_iter()
            .map(|row| PlainRow::new(row).into_element());
        page::page(page::header(spec::status(self.count), Spinner), rows)
    }
}

/// The turning square. It is its own component, so each animation frame renders it alone.
#[derive(PartialEq)]
struct Spinner;

impl Component for Spinner {
    fn render(&self) -> impl IntoElement {
        // AnimNum eases linearly by default; restarting at the end makes it turn forever.
        let animation = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            conf.on_finish(OnFinish::restart());
            AnimNum::new(0.0, 360.0).time(spec::SPINNER_PERIOD_MS)
        });
        rect()
            .width(Size::px(spec::SPINNER_SIZE))
            .height(Size::px(spec::SPINNER_SIZE))
            .corner_radius(spec::SPINNER_RADIUS)
            .background(page::color(spec::SPINNER_COLOR))
            .rotate(animation.get().value())
    }
}
