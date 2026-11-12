//! The dashboard scenario: the analytics dashboard of `apps/shared/dashboard.rs`. One state
//! struct holds what the controls change, and every pass builds the whole page from it (egui's
//! immediate mode): a left panel for the sidebar, a top panel for the top bar, and a central panel
//! with one vertical scroll area of cards.

mod chart;
mod filters;
mod kpis;
mod orders;
mod sidebar;
#[path = "../../../shared/dashboard.rs"]
pub mod spec;
mod style;
mod switch;
mod topbar;

use eframe::egui::{self, Align, CentralPanel, Frame, Layout, Margin, ScrollArea, Ui, vec2};

use self::spec::{Order, Status};
use crate::window;

/// Opens the dashboard window with `count` orders and runs it until it closes.
pub fn run(title: &str, count: usize) -> eframe::Result {
    window::run(title, style::setup, move || Box::new(Dashboard::new(count)))
}

/// What the controls of the dashboard change.
struct Dashboard {
    orders: Vec<Order>,
    /// The active nav item.
    nav: usize,
    /// The text of the search input.
    search: String,
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
            orders: spec::orders(count),
            nav: 0,
            search: String::new(),
            segment: 0,
            compare: false,
            shown: [true; Status::ALL.len()],
        }
    }

    /// Checks every status, turns compare off, selects the first segment and clears the search.
    fn reset(&mut self) {
        self.shown = [true; Status::ALL.len()];
        self.compare = false;
        self.segment = 0;
        self.search.clear();
    }

    /// Shows the content: the metric cards, the chart and filter cards, and the orders.
    fn content(&mut self, ui: &mut Ui) {
        kpis::show(ui);
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
        orders::show(ui, &self.orders, &self.shown, &self.search);
    }
}

impl eframe::App for Dashboard {
    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        sidebar::show(ui, &mut self.nav);
        topbar::show(ui, &mut self.search);
        let page = Frame::new().fill(style::color(spec::PAGE_BG));
        CentralPanel::default().frame(page).show(ui, |ui| {
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                Frame::new()
                    .inner_margin(Margin::same(spec::CONTENT_PADDING as i8))
                    .show(ui, |ui| self.content(ui));
            });
        });
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        style::color(spec::PAGE_BG).to_normalized_gamma_f32()
    }
}
