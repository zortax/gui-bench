//! The content of the scroll view: the page of the active nav item.

use std::rc::Rc;

use freya::prelude::*;

use super::chart::Chart;
use super::filters::Filters;
use super::kpis::Kpis;
use super::messages::Messages;
use super::orders::Orders;
use super::spec::{self, Page};
use super::text;

/// The page of the active nav item. It reads the page, so a nav click that changes the page
/// drops the elements of the old page and builds the new one.
#[derive(PartialEq)]
pub struct PageView {
    pub count: usize,
    pub orders: Rc<Vec<spec::Order>>,
    pub active: State<usize>,
    pub tick: State<u64>,
    pub search: State<String>,
    pub series: State<usize>,
    pub compare: State<bool>,
    pub shown: State<[bool; 4]>,
    pub sorted: State<bool>,
}

impl Component for PageView {
    fn render(&self) -> impl IntoElement {
        let active = self.active;
        let page = use_memo(move || spec::page(active()));
        match page() {
            Page::Overview => self.overview().into_element(),
            Page::Messages => Messages { count: self.count }.into_element(),
            Page::Empty => empty().into_element(),
        }
    }
}

impl PageView {
    /// Returns the overview: the metric cards, the chart beside the filters, and the orders.
    fn overview(&self) -> Rect {
        rect()
            .width(Size::fill())
            .padding(spec::CONTENT_PADDING)
            .spacing(spec::GAP)
            .child(Kpis { tick: self.tick })
            .child(
                rect()
                    .horizontal()
                    .content(Content::Flex)
                    .width(Size::fill())
                    .height(Size::px(spec::ROW2_HEIGHT))
                    .spacing(spec::GAP)
                    .child(Chart {
                        series: self.series,
                        compare: self.compare,
                    })
                    .child(Filters {
                        shown: self.shown,
                        compare: self.compare,
                        series: self.series,
                        search: self.search,
                        sorted: self.sorted,
                    }),
            )
            .child(Orders {
                orders: self.orders.clone(),
                shown: self.shown,
                search: self.search,
                sorted: self.sorted,
            })
    }
}

/// Returns the empty page: the label centered across the content.
fn empty() -> Rect {
    rect()
        .width(Size::fill())
        .padding(Gaps::new(spec::EMPTY_TOP, 0.0, 0.0, 0.0))
        .cross_align(Alignment::Center)
        .child(text(
            spec::EMPTY_LABEL,
            spec::FONT_SIZE,
            FontWeight::NORMAL,
            spec::MUTED,
        ))
}
