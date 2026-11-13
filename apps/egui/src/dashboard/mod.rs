//! The app scenario: the analytics app of `apps/shared/dashboard.rs`. One state struct holds
//! what the controls change, and every pass builds the whole window from it (egui's immediate
//! mode): a left panel for the sidebar, a top panel for the top bar, and a central panel with one
//! vertical scroll area that shows the page of the active nav item. The list scenario is `list`.

mod chart;
mod filters;
mod kpis;
pub mod list;
mod live;
mod messages;
mod orders;
mod sidebar;
#[path = "../../../shared/dashboard.rs"]
pub mod spec;
mod style;
mod switch;
mod topbar;

use eframe::egui::{self, Align, CentralPanel, Frame, Layout, Margin, ScrollArea, Ui, vec2};

use self::live::Live;
use self::orders::Orders;
use self::spec::{Message, Page, Status};
use self::style::{Weight, text};
use crate::window;

/// The space between the top of the content and the label of the empty page.
const EMPTY_SPACE: f32 = spec::EMPTY_TOP - spec::CONTENT_PADDING;

/// Opens the app window with `count` orders and `count` messages and runs it until it closes.
pub fn run(title: &str, count: usize) -> eframe::Result {
    window::run(title, style::setup, move || Box::new(Dashboard::new(count)))
}

/// What the controls of the app change.
struct Dashboard {
    orders: Orders,
    messages: Vec<Message>,
    /// The active nav item, which picks the page.
    nav: usize,
    /// The text of the search input.
    search: String,
    /// The live updates of the metric cards.
    live: Live,
    /// The selected segment of the chart, which is the series it shows.
    segment: usize,
    /// Whether the chart shows the previous period beside the current one.
    compare: bool,
    /// Whether the orders of `Status::ALL[k]` are shown.
    shown: [bool; 4],
}

impl Dashboard {
    fn new(count: usize) -> Self {
        Self {
            orders: Orders::new(count),
            messages: spec::messages(count),
            nav: 0,
            search: String::new(),
            live: Live::default(),
            segment: 0,
            compare: false,
            shown: [true; Status::ALL.len()],
        }
    }

    /// Checks every status, turns compare off, selects the first segment, clears the search and
    /// brings the order of the data back.
    fn reset(&mut self) {
        self.shown = [true; Status::ALL.len()];
        self.compare = false;
        self.segment = 0;
        self.search.clear();
        self.orders.sort(false);
    }

    /// Shows the overview: the metric cards, the chart and filter cards, and the orders.
    fn overview(&mut self, ui: &mut Ui) {
        kpis::show(ui, self.live.tick(ui));
        ui.add_space(spec::GAP);
        // The filter card first, from the right: the chart card takes the rest of the width, and
        // the switch and Reset change the chart before it is laid out.
        let row = vec2(ui.available_width(), spec::ROW2_HEIGHT);
        ui.allocate_ui_with_layout(row, Layout::right_to_left(Align::Min), |ui| {
            let filters = vec2(spec::FILTERS_WIDTH, spec::ROW2_HEIGHT);
            ui.allocate_ui_with_layout(filters, Layout::top_down(Align::Min), |ui| {
                if filters::show(ui, &mut self.shown, &mut self.compare) {
                    self.reset();
                }
            });
            ui.add_space(spec::GAP);
            let chart = vec2(ui.available_width(), spec::ROW2_HEIGHT);
            ui.allocate_ui_with_layout(chart, Layout::top_down(Align::Min), |ui| {
                chart::show(ui, &mut self.segment, self.compare);
            });
        });
        ui.add_space(spec::GAP);
        self.orders.show(ui, &self.shown, &self.search);
    }
}

/// Shows the empty page: its label, centered across the content.
fn empty(ui: &mut Ui) {
    ui.add_space(EMPTY_SPACE);
    ui.vertical_centered(|ui| {
        ui.label(text(
            spec::EMPTY_LABEL,
            spec::FONT_SIZE,
            Weight::Regular,
            spec::MUTED,
        ));
    });
}

impl eframe::App for Dashboard {
    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        sidebar::show(ui, &mut self.nav);
        let title = spec::NAV_ITEMS[self.nav];
        topbar::show(ui, title, &mut self.search, &mut self.live);
        let page = Frame::new().fill(style::color(spec::PAGE_BG));
        CentralPanel::default().frame(page).show(ui, |ui| {
            // Every page scrolls on its own.
            let scroll = ScrollArea::vertical().id_salt(self.nav).auto_shrink(false);
            scroll.show(ui, |ui| {
                Frame::new()
                    .inner_margin(Margin::same(spec::CONTENT_PADDING as i8))
                    .show(ui, |ui| match spec::page(self.nav) {
                        Page::Overview => self.overview(ui),
                        Page::Messages => messages::show(ui, &self.messages),
                        Page::Empty => empty(ui),
                    });
            });
        });
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        style::color(spec::PAGE_BG).to_normalized_gamma_f32()
    }
}
