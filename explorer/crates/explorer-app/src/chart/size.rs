//! The width of an element, kept current by a ResizeObserver. It starts at a hint, so the server
//! renders a chart at a likely width and the browser hydrates the same markup before it measures.

use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

/// Returns the content width of a div, starting at `hint` and updated on every resize.
pub fn use_width(node: NodeRef<html::Div>, hint: f64) -> ReadSignal<f64> {
    let (width, set_width) = signal(hint);
    Effect::new(move |_| {
        let Some(el) = node.get() else {
            return;
        };
        let w = f64::from(el.client_width());
        if w > 0.0 && (w - width.get_untracked()).abs() >= 1.0 {
            set_width.set(w);
        }
        let callback = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
            if let Ok(entry) = entries.get(0).dyn_into::<web_sys::ResizeObserverEntry>() {
                let w = entry.content_rect().width();
                if (w - width.get_untracked()).abs() >= 1.0 {
                    set_width.set(w);
                }
            }
        });
        let Ok(observer) = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) else {
            return;
        };
        observer.observe(&el);
        let keep = SendWrapper::new((observer, callback));
        on_cleanup(move || keep.0.disconnect());
    });
    width
}
