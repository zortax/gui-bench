//! The app scenario (`apps/shared/dashboard.rs`): a sidebar, a top bar, and a scroll container
//! with the page of the active nav item: the overview (the metric cards, the chart and filter
//! cards, and the table of `count` orders), the messages feed, or an empty page. The sidebar,
//! the top bar, each page, the metric cards and the orders are views of their own; the filter
//! and live states are entities that the views observe. The content scrolls as GPUI's
//! `overflow_y_scroll` does by default.
//!
//! The `tuned` feature embeds the sidebar, the top bar, the metric cards and the orders as
//! cached views (`style::embed`); the default build embeds every view plainly.
//!
//! The list scenario (`list.rs`) shows the orders alone in GPUI's `uniform_list`.

mod chart;
mod filters;
mod kpis;
mod list;
mod live;
mod messages;
mod orders;
mod overview;
mod sidebar;
#[path = "../../../shared/dashboard.rs"]
pub mod spec;
mod style;
mod topbar;

use gpui_kit::component::input::InputState;
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::*;

use self::filters::Filters;
pub use self::list::OrderList;
use self::live::Live;
use self::messages::Messages;
use self::overview::Overview;
use self::sidebar::{Selected, Sidebar};
use self::spec::Page;
pub use self::style::apply_theme;
use self::style::embed;
use self::topbar::TopBar;

/// The view of the page that the content shows.
enum Content {
    Overview(Entity<Overview>),
    Messages(Entity<Messages>),
    Empty,
}

/// The dashboard.
pub struct Dashboard {
    /// The number of orders and of messages.
    count: usize,
    sidebar: Entity<Sidebar>,
    topbar: Entity<TopBar>,
    search: Entity<InputState>,
    filters: Entity<Filters>,
    live: Entity<Live>,
    page: Page,
    content: Content,
    _nav: Subscription,
}

impl Dashboard {
    pub fn new(count: usize, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder(spec::SEARCH_PLACEHOLDER));
        let filters = cx.new(|_| Filters::default());
        let live = cx.new(|_| Live::default());
        let sidebar = cx.new(|_| Sidebar::default());
        let nav = cx.subscribe(&sidebar, |this, _, selected: &Selected, cx| {
            this.show(selected.0, cx)
        });
        let overview =
            cx.new(|cx| Overview::new(count, filters.clone(), search.clone(), live.clone(), cx));
        Self {
            count,
            sidebar,
            topbar: cx.new(|_| TopBar::new(search.clone(), live.clone())),
            search,
            filters,
            live,
            page: Page::Overview,
            content: Content::Overview(overview),
            _nav: nav,
        }
    }

    /// Shows the page of nav item `item`: drops the view of the old page and makes the new one.
    fn show(&mut self, item: usize, cx: &mut Context<Self>) {
        self.topbar
            .update(cx, |topbar, cx| topbar.set_active(item, cx));
        let page = spec::page(item);
        if page == self.page {
            return;
        }
        self.page = page;
        self.content = match page {
            Page::Overview => Content::Overview(cx.new(|cx| {
                Overview::new(
                    self.count,
                    self.filters.clone(),
                    self.search.clone(),
                    self.live.clone(),
                    cx,
                )
            })),
            Page::Messages => Content::Messages(cx.new(|_| Messages::new(self.count))),
            Page::Empty => Content::Empty,
        };
        cx.notify();
    }
}

/// Returns the empty page: its label, centered across the content.
fn empty() -> Div {
    v_flex()
        .w_full()
        .p(px(spec::CONTENT_PADDING))
        .items_center()
        .child(
            div()
                .mt(px(spec::EMPTY_TOP - spec::CONTENT_PADDING))
                .text_color(rgb(spec::MUTED))
                .child(spec::EMPTY_LABEL),
        )
}

impl Render for Dashboard {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let page = match &self.content {
            Content::Overview(overview) => overview.clone().into_any_element(),
            Content::Messages(messages) => messages.clone().into_any_element(),
            Content::Empty => empty().into_any_element(),
        };
        let sidebar = StyleRefinement::default()
            .flex_none()
            .w(px(spec::SIDEBAR_WIDTH))
            .h_full();
        let topbar = StyleRefinement::default()
            .flex_none()
            .w_full()
            .h(px(spec::TOPBAR_HEIGHT));
        h_flex()
            .size_full()
            .bg(rgb(spec::PAGE_BG))
            .text_size(px(spec::FONT_SIZE))
            .text_color(rgb(spec::TEXT))
            .child(embed(&self.sidebar, sidebar))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(embed(&self.topbar, topbar))
                    .child(
                        div()
                            .id("content")
                            .flex_1()
                            .min_h_0()
                            .w_full()
                            .overflow_y_scroll()
                            .child(page),
                    ),
            )
    }
}
