//! The ticker timer: a foreground task that waits on the executor's timer and then updates the
//! view, for as long as the view lives and holds the task.

use std::time::Duration;

use gpui_kit::*;

/// Starts a task that calls `tick` on the view every `interval_ms`.
pub fn every<V: 'static>(
    interval_ms: u64,
    cx: &mut Context<V>,
    tick: fn(&mut V, &mut Context<V>),
) -> Task<()> {
    let interval = Duration::from_millis(interval_ms);
    cx.spawn(async move |this, cx| {
        loop {
            cx.background_executor().timer(interval).await;
            if this.update(cx, tick).is_err() {
                break;
            }
        }
    })
}
