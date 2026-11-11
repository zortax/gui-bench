//! A time stamp: UTC on the server, the local time of the browser once the page runs.

use explorer_core::units::date_time;
use leptos::prelude::*;

/// Unix seconds as a date and time.
#[component]
pub fn LocalTime(unix: u64) -> impl IntoView {
    let offset = RwSignal::new(None::<i64>);
    Effect::new(move |_| {
        let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(unix as f64 * 1000.0));
        offset.set(Some(-(date.get_timezone_offset() as i64)));
    });
    view! {
        <time title=move || if offset.get().is_some() { "local time" } else { "UTC" }>
            {move || match offset.get() {
                Some(o) => date_time(unix, o),
                None => format!("{} UTC", date_time(unix, 0)),
            }}
        </time>
    }
}
