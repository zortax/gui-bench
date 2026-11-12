//! The dashboard scenario (`apps/shared/dashboard.rs`): a sidebar, a top bar, and a scroll
//! container with the metric cards, the chart and filter cards, and the table of `count` orders.
//! The root view holds the chart and filter state and the search input; the sidebar and the
//! orders are views of their own. The content scrolls as GPUI's `overflow_y_scroll` does by
//! default.

mod chart;
mod filters;
mod kpis;
mod orders;
mod sidebar;
#[path = "../../../shared/dashboard.rs"]
mod spec;
mod style;
mod topbar;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::*;

use self::orders::Orders;
use self::sidebar::Sidebar;

/// The dashboard.
pub struct Dashboard {
    sidebar: Entity<Sidebar>,
    search: Entity<InputState>,
    orders: Entity<Orders>,
    /// The chart series, an index into `SEGMENTS`.
    series: usize,
    /// Whether the chart shows the previous period.
    compare: bool,
    /// Whether the orders of `Status::ALL[k]` show.
    shown: [bool; 4],
    _search_changes: Subscription,
}

impl Dashboard {
    pub fn new(count: usize, window: &mut Window, cx: &mut Context<Self>) -> Self {
        style::apply_theme(cx);
        let search = cx.new(|cx| InputState::new(window, cx).placeholder(spec::SEARCH_PLACEHOLDER));
        let search_changes = cx.subscribe(&search, |this, _, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                this.filter(cx);
            }
        });
        Self {
            sidebar: cx.new(|_| Sidebar::default()),
            search,
            orders: cx.new(|_| Orders::new(count)),
            series: 0,
            compare: false,
            shown: [true; 4],
            _search_changes: search_changes,
        }
    }

    /// Passes the status filter and the search text to the orders.
    fn filter(&mut self, cx: &mut Context<Self>) {
        let search = self.search.read(cx).value();
        let shown = self.shown;
        self.orders
            .update(cx, |orders, cx| orders.filter(&shown, &search, cx));
    }

    /// Shows or hides the orders of status `k`.
    fn show_status(&mut self, k: usize, shown: bool, cx: &mut Context<Self>) {
        self.shown[k] = shown;
        self.filter(cx);
        cx.notify();
    }

    /// Turns the previous period of the chart on or off.
    fn set_compare(&mut self, compare: bool, cx: &mut Context<Self>) {
        self.compare = compare;
        cx.notify();
    }

    /// Shows the series of the checked segment. A click on the selected segment unchecks it in
    /// the group, and the selection stays.
    fn select_series(&mut self, checked: &[bool], cx: &mut Context<Self>) {
        if let Some(series) = checked.iter().position(|checked| *checked) {
            self.series = series;
            cx.notify();
        }
    }

    /// Checks every status, turns compare off, selects the first segment and clears the search.
    fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.shown = [true; 4];
        self.compare = false;
        self.series = 0;
        self.search
            .update(cx, |search, cx| search.set_value("", window, cx));
        self.filter(cx);
        cx.notify();
    }
}

impl Render for Dashboard {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chart = chart::chart(
            self.series,
            self.compare,
            cx.listener(|this, checked: &Vec<bool>, _, cx| this.select_series(checked, cx)),
        );
        let blocks = v_flex()
            .id("blocks")
            .w_full()
            .p(px(spec::CONTENT_PADDING))
            .gap(px(spec::GAP))
            .child(kpis::kpis())
            .child(
                h_flex()
                    .flex_none()
                    .w_full()
                    .h(px(spec::ROW2_HEIGHT))
                    .gap(px(spec::GAP))
                    .child(chart)
                    .child(filters::filters(self, cx)),
            )
            .child(self.orders.clone());
        h_flex()
            .size_full()
            .bg(rgb(spec::PAGE_BG))
            .text_size(px(spec::FONT_SIZE))
            .text_color(rgb(spec::TEXT))
            .child(
                self.sidebar.clone().cached(
                    StyleRefinement::default()
                        .flex_none()
                        .w(px(spec::SIDEBAR_WIDTH))
                        .h_full(),
                ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(topbar::topbar(&self.search))
                    .child(
                        div()
                            .id("content")
                            .flex_1()
                            .min_h_0()
                            .w_full()
                            .overflow_y_scroll()
                            .child(blocks),
                    ),
            )
    }
}
