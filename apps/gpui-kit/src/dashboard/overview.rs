//! The overview page: the metric cards, the chart and filter cards, and the orders. The page
//! draws the chart and the filter card from the filter state; the metric cards and the orders
//! are views of their own.

use gpui_kit::component::input::InputState;
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::*;

use super::filters::{self, Filters};
use super::kpis::Kpis;
use super::live::Live;
use super::orders::Orders;
use super::style::embed;
use super::{chart, spec};

/// The overview page.
pub struct Overview {
    kpis: Entity<Kpis>,
    orders: Entity<Orders>,
    filters: Entity<Filters>,
    search: Entity<InputState>,
    _filter_changes: Subscription,
}

impl Overview {
    pub fn new(
        count: usize,
        filters: Entity<Filters>,
        search: Entity<InputState>,
        live: Entity<Live>,
        cx: &mut Context<Self>,
    ) -> Self {
        let filter_changes = cx.observe(&filters, |_, _, cx| cx.notify());
        Self {
            kpis: cx.new(|cx| Kpis::new(live, cx)),
            orders: cx.new(|cx| Orders::new(count, filters.clone(), search.clone(), cx)),
            filters,
            search,
            _filter_changes: filter_changes,
        }
    }

    /// Clears the search and brings the filters back to the startup state.
    pub fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.search
            .update(cx, |search, cx| search.set_value("", window, cx));
        self.filters.update(cx, |filters, cx| filters.reset(cx));
    }
}

impl Render for Overview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (series, compare) = {
            let filters = self.filters.read(cx);
            (filters.series, filters.compare)
        };
        let chart = chart::chart(series, compare, {
            let filters = self.filters.clone();
            move |checked: &Vec<bool>, _: &mut Window, cx: &mut App| {
                filters.update(cx, |filters, cx| filters.select_series(checked, cx))
            }
        });
        let kpis = StyleRefinement::default()
            .flex_none()
            .w_full()
            .h(px(spec::KPI_HEIGHT));
        let orders = StyleRefinement::default()
            .flex_none()
            .w_full()
            .h(px(self.orders.read(cx).height()));
        v_flex()
            .w_full()
            .p(px(spec::CONTENT_PADDING))
            .gap(px(spec::GAP))
            .child(embed(&self.kpis, kpis))
            .child(
                h_flex()
                    .flex_none()
                    .w_full()
                    .h(px(spec::ROW2_HEIGHT))
                    .gap(px(spec::GAP))
                    .child(chart)
                    .child(filters::filters(&self.filters, cx)),
            )
            .child(embed(&self.orders, orders))
    }
}
