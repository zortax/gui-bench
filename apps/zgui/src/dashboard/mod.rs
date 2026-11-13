//! The app scenario: a sidebar with nav, a top bar with the search, the live switch and the
//! export menu, and one scroll container with the page of the active nav item: the overview (the
//! metric cards, the revenue chart, the filters and the orders table), the messages feed, or an
//! empty page (`dashboard.rs`).
//!
//! The controls are `zgui-ui` components under one `ThemeProvider`, whose tokens take the colors
//! of the spec; everything else is plain elements styled by `dashboard.css`.

mod chart;
mod filters;
mod kpis;
mod list;
mod live;
mod messages;
mod orders;
mod sidebar;
#[path = "../../../shared/dashboard.rs"]
pub mod spec;
mod topbar;

use zgui::prelude::*;
use zgui::reactive::LocalStorage;
use zgui_ui::prelude::*;
use zgui_ui_tokens::prelude::*;

use self::chart::RevenueChartProps;
use self::filters::FilterCardProps;
use self::kpis::MetricCardsProps;
pub use self::list::OrderListProps;
use self::live::Live;
use self::messages::MessageFeedProps;
use self::orders::OrdersTableProps;
use self::sidebar::NavSidebarProps;
use self::spec::Page;
use self::topbar::TopBarProps;

/// The look of the whole dashboard.
pub const SHEET: &str = include_str!("dashboard.css");

/// The tokens of the component library that differ from its default light theme.
pub(self) const TOKENS: &str = "
    --zui-color-primary: #4f46e5;
    --zui-color-primary-foreground: #ffffff;
    --zui-color-foreground: #111827;
    --zui-color-muted-foreground: #6b7280;
    --zui-color-input: #d1d5db;
    --zui-color-border: #d1d5db;
    --zui-color-accent: #f3f4f6;
    --zui-color-accent-foreground: #111827;
    --zui-color-ring: #6366f1;
    --zui-type-family-sans: sans-serif;
";

/// The values the controls write and the chart and the orders read.
#[derive(Clone, Copy)]
pub struct Filters {
    /// The search text.
    pub search: RwSignal<String, LocalStorage>,
    /// Whether each status of `Status::ALL` is shown.
    pub shown: [RwSignal<Checked, LocalStorage>; 4],
    /// Whether the chart shows the previous period too.
    pub compare: RwSignal<bool, LocalStorage>,
    /// The selected segment, an index of `SEGMENTS`.
    pub segment: RwSignal<usize, LocalStorage>,
    /// Whether the orders are sorted by amount.
    pub sorted: RwSignal<bool>,
}

impl Filters {
    /// The startup state: no search, every status, no compare, the first segment.
    fn new() -> Self {
        Self {
            search: RwSignal::new_local(String::new()),
            shown: std::array::from_fn(|_| RwSignal::new_local(Checked::Yes)),
            compare: RwSignal::new_local(false),
            segment: RwSignal::new_local(0),
            sorted: RwSignal::new(false),
        }
    }

    /// Goes back to the startup state.
    pub fn reset(&self) {
        for shown in self.shown {
            shown.set(Checked::Yes);
        }
        self.compare.set(false);
        self.segment.set(0);
        self.search.set(String::new());
        self.sorted.set(false);
    }

    /// Reads which statuses are shown, tracked.
    pub fn statuses(&self) -> [bool; 4] {
        self.shown.map(|shown| shown.get().is_checked())
    }
}

/// The dashboard with `count` orders and `count` messages.
#[component]
pub fn Dashboard(count: usize) -> impl IntoView {
    let filters = Filters::new();
    let active = RwSignal::new(0_usize);
    let live = Live::new();
    let page = Memo::new(move |_| spec::page(active.get()));
    view! {
        ThemeProvider(light = Theme::light().with_css(TOKENS), scheme = ColorScheme::Light) {
            row(class = "dash") {
                NavSidebar(active = active)
                column(class = "main") {
                    TopBar(filters = filters, active = active, live = live)
                    scroll(class = "content") {
                        if move || page.get() == Page::Overview {
                            column(class = "content__body") {
                                MetricCards(live = live)
                                row(class = "row2") {
                                    RevenueChart(filters = filters)
                                    FilterCard(filters = filters)
                                }
                                OrdersTable(count = count, filters = filters)
                            }
                        } else {
                            if move || page.get() == Page::Messages {
                                MessageFeed(count = count)
                            } else {
                                column(class = "content__body empty") {
                                    text(class = "empty__label") {{spec::EMPTY_LABEL}}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Returns a spec color as CSS.
pub fn hex(color: u32) -> String {
    format!("#{color:06x}")
}
