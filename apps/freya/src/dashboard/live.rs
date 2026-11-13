//! The live updates: while the switch of the top bar is on, a timer advances the tick that the
//! metric cards show, and the spinner turns.

use std::time::Duration;

use freya::animation::*;
use freya::prelude::*;

use super::{color, spec};

/// Returns the tick of the metric values: 0 while `live` is off, and one more every
/// `LIVE_INTERVAL_MS` while it is on. The future reads `live`, so a change of the switch
/// restarts it.
pub fn use_live(live: State<bool>) -> State<u64> {
    let mut tick = use_state(|| 0u64);
    use_future(move || {
        let on = live();
        async move {
            tick.set_if_modified(0);
            while on {
                timer(Duration::from_millis(spec::LIVE_INTERVAL_MS)).await;
                *tick.write() += 1;
            }
        }
    });
    tick
}

/// The place of the spinner in the top bar, which holds the spinner while live is on.
#[derive(PartialEq)]
pub struct LiveSpinner {
    pub live: State<bool>,
}

impl Component for LiveSpinner {
    fn render(&self) -> impl IntoElement {
        rect()
            .width(Size::px(spec::SPINNER_SIZE))
            .height(Size::px(spec::SPINNER_SIZE))
            .maybe_child((self.live)().then_some(Spinner))
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
            .background(color(spec::PRIMARY))
            .rotate(animation.get().value())
    }
}
