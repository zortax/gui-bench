//! Zoom and pan of a plot: the viewport of zoomed domains, a non-passive wheel listener (so
//! ctrl+wheel zooms the chart instead of the page), and pointer positions inside an element.

use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

/// The zoomed domains of a plot; `None` shows the full data range.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Viewport {
    pub x: Option<(f64, f64)>,
    pub y: Option<(f64, f64)>,
}

impl Viewport {
    pub fn is_zoomed(&self) -> bool {
        self.x.is_some() || self.y.is_some()
    }
}

/// A drag in progress, in pixels relative to the plot element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drag {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    /// The drag pans; otherwise it selects a box to zoom into.
    pub pan: bool,
}

/// Returns the pointer position of a mouse event relative to an element.
pub fn local(el: &web_sys::Element, ev: &web_sys::MouseEvent) -> (f64, f64) {
    let rect = el.get_bounding_client_rect();
    (
        f64::from(ev.client_x()) - rect.left(),
        f64::from(ev.client_y()) - rect.top(),
    )
}

/// Calls `f` on wheel events over the div with ctrl or meta held, and stops the page from
/// zooming or scrolling for those events. Plain wheel events scroll the page.
pub fn on_zoom_wheel(node: NodeRef<html::Div>, f: impl Fn(web_sys::WheelEvent) + 'static) {
    let f = SendWrapper::new(std::rc::Rc::new(f));
    Effect::new(move |_| {
        let Some(el) = node.get() else {
            return;
        };
        let f = f.clone();
        let callback =
            Closure::<dyn FnMut(web_sys::WheelEvent)>::new(move |ev: web_sys::WheelEvent| {
                if ev.ctrl_key() || ev.meta_key() {
                    ev.prevent_default();
                    f(ev);
                }
            });
        let options = web_sys::AddEventListenerOptions::new();
        options.set_passive(false);
        let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
            "wheel",
            callback.as_ref().unchecked_ref(),
            &options,
        );
        let keep = SendWrapper::new((el.clone(), callback));
        on_cleanup(move || {
            let (el, cb) = &*keep;
            let _ = el.remove_event_listener_with_callback("wheel", cb.as_ref().unchecked_ref());
        });
    });
}
