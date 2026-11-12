//! The filter card: a built-in `Checkbox` per status, the built-in `Switch` for compare, and the
//! Reset (outline variant) and Apply buttons. The controls show the signals of the state and emit
//! events; Apply does nothing.

use vizia::prelude::*;

use super::{Dashboard, DashboardEvent};
use crate::dashboard_spec::{APPLY_LABEL, COMPARE_LABEL, FILTERS_TITLE, RESET_LABEL, Status};

/// Builds the filter card.
pub fn build(cx: &mut Context, state: Dashboard) {
    VStack::new(cx, move |cx| {
        Label::new(cx, FILTERS_TITLE)
            .class("card-title")
            .class("filters-title");
        VStack::new(cx, move |cx| {
            VStack::new(cx, move |cx| {
                for (k, status) in Status::ALL.into_iter().enumerate() {
                    HStack::new(cx, move |cx| {
                        Checkbox::new(cx, state.shown[k])
                            .on_toggle(move |cx| cx.emit(DashboardEvent::ToggleStatus(k)));
                        Label::new(cx, status.label());
                    })
                    .class("check-row");
                }
            })
            .class("checks");
            HStack::new(cx, move |cx| {
                Label::new(cx, COMPARE_LABEL);
                Switch::new(cx, state.compare)
                    .on_toggle(|cx| cx.emit(DashboardEvent::ToggleCompare));
            })
            .class("compare-row");
        })
        .class("filters-body");
        HStack::new(cx, |cx| {
            Button::new(cx, |cx| Label::new(cx, RESET_LABEL))
                .variant(ButtonVariant::Outline)
                .on_press(|cx| cx.emit(DashboardEvent::Reset));
            Button::new(cx, |cx| Label::new(cx, APPLY_LABEL)).class("primary");
        })
        .class("filter-buttons");
    })
    .class("card")
    .class("filters");
}
