//! The orders card: a header of column titles, then one row per order that passes the filters
//! and the search, all of them laid out every pass (no virtualization). A row is a frame of
//! fixed-width cells, its fill taken from its response, and a separator under it. A click on the
//! amount title sorts the orders.

use eframe::egui::{
    Align, CornerRadius, Frame, Layout, ProgressBar, Sense, Separator, Stroke, Ui, vec2,
};

use super::spec::{self, Order};
use super::style::{Weight, area, avatar, card, color, pill, text};

/// The cells of a row before the amount, from the left of the card: each from its column's left
/// edge to the next one. The amount, right-aligned, goes from the end of the progress bar to its
/// own end.
const CELLS: [(f32, f32); 4] = [
    (spec::COL_ORDER, spec::COL_CUSTOMER),
    (spec::COL_CUSTOMER, spec::COL_STATUS),
    (spec::COL_STATUS, spec::COL_PROGRESS),
    (
        spec::COL_PROGRESS,
        spec::COL_PROGRESS + spec::PROGRESS_WIDTH,
    ),
];
/// The space between the avatar and the name.
const NAME_GAP: f32 = 12.0;

/// The orders of the card, in the order the table shows them.
pub struct Orders {
    count: usize,
    orders: Vec<Order>,
    /// Whether the orders are sorted by amount.
    sorted: bool,
}

impl Orders {
    /// `count` orders in the order of the data.
    pub fn new(count: usize) -> Self {
        Self {
            count,
            orders: spec::orders(count),
            sorted: false,
        }
    }

    /// Sorts the orders by amount, or brings the order of the data back.
    pub fn sort(&mut self, sorted: bool) {
        if self.sorted != sorted {
            self.sorted = sorted;
            self.orders = spec::arranged(spec::orders(self.count), sorted);
        }
    }

    /// Shows the orders card with the orders that pass `shown` and `search`.
    pub fn show(&mut self, ui: &mut Ui, shown: &[bool; 4], search: &str) {
        card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            if header(ui, spec::CARD_RADIUS as u8, spec::COL_AMOUNT_END, self.sorted) {
                self.sort(!self.sorted);
            }
            rows_style(ui);
            let mut visible = self
                .orders
                .iter()
                .filter(|order| spec::visible(order, shown, search))
                .peekable();
            while let Some(order) = visible.next() {
                row(ui, order, spec::COL_AMOUNT_END, visible.peek().is_none());
            }
        });
    }
}

/// Sets what every row shares: the progress track color and the color of the rule under a row.
pub fn rows_style(ui: &mut Ui) {
    ui.visuals_mut().extreme_bg_color = color(spec::TRACK);
    set_rule(ui, spec::ROW_RULE);
}

/// The header: the column titles on the header fill with `radius` at its top corners, and the
/// rule under it. The amount column ends `end` from the left. Returns true when the amount title
/// was clicked; it is in the primary color while the orders are `sorted`.
pub fn header(ui: &mut Ui, radius: u8, end: f32, sorted: bool) -> bool {
    let height = spec::TABLE_HEADER_HEIGHT - 1.0;
    let mut clicked = false;
    Frame::new()
        .fill(color(spec::TABLE_HEADER_BG))
        .corner_radius(CornerRadius {
            nw: radius,
            ne: radius,
            sw: 0,
            se: 0,
        })
        .show(ui, |ui| {
            cells(ui, height, end, |ui, column| {
                if column == spec::SORT_COLUMN {
                    let title_color = if sorted { spec::PRIMARY } else { spec::MUTED };
                    let title = text(spec::COLUMNS[column], 12.0, Weight::Semibold, title_color);
                    let size = vec2(spec::SORT_WIDTH, height);
                    let sort = area(ui, size, Layout::right_to_left(Align::Center), |ui| {
                        ui.label(title);
                    });
                    clicked = sort.response.interact(Sense::click()).clicked();
                } else {
                    let title = text(spec::COLUMNS[column], 12.0, Weight::Semibold, spec::MUTED);
                    ui.label(title);
                }
            });
        });
    set_rule(ui, spec::RULE);
    ui.add(Separator::default().spacing(1.0));
    clicked
}

/// One order row, its amount ending `end` from the left; the `last` one takes the card's bottom
/// corners for its hover fill.
pub fn row(ui: &mut Ui, order: &Order, end: f32, last: bool) {
    ui.push_id(order.number, |ui| {
        let radius = if last { spec::CARD_RADIUS as u8 } else { 0 };
        let mut frame = Frame::new()
            .corner_radius(CornerRadius {
                nw: 0,
                ne: 0,
                sw: radius,
                se: radius,
            })
            .begin(ui);
        cells(
            &mut frame.content_ui,
            spec::ORDER_ROW_HEIGHT - 1.0,
            end,
            |ui, column| match column {
                0 => {
                    let number = text(order.number_text(), 14.0, Weight::Regular, spec::MUTED);
                    ui.label(number);
                }
                1 => {
                    let initials = text(&order.initials, 11.0, Weight::Semibold, 0xffffff);
                    avatar(ui, spec::AVATAR, order.avatar, initials);
                    ui.add_space(NAME_GAP);
                    let name = text(&order.customer, 14.0, Weight::Regular, spec::TEXT);
                    ui.label(name);
                }
                2 => {
                    let (bg, fg) = order.status.colors();
                    let label = text(order.status.label(), 12.0, Weight::Semibold, fg);
                    pill(ui, label, bg, 22.0, 10.0);
                }
                3 => progress(ui, order.progress),
                _ => {
                    let amount = text(order.amount(), 14.0, Weight::Semibold, spec::TEXT);
                    ui.label(amount);
                }
            },
        );
        let response = frame.allocate_space(ui);
        if response.hovered() {
            frame.frame.fill = color(spec::ROW_HOVER);
        }
        frame.paint(ui);
        ui.add(Separator::default().spacing(1.0));
    });
}

/// Lays out the cells of a row `height` tall whose amount cell ends `end` from the left; `add`
/// adds the content of cell `column`.
fn cells(ui: &mut Ui, height: f32, end: f32, mut add: impl FnMut(&mut Ui, usize)) {
    let size = vec2(ui.available_width(), height);
    area(ui, size, Layout::left_to_right(Align::Center), |ui| {
        ui.add_space(CELLS[0].0);
        for (column, (start, end)) in CELLS.into_iter().enumerate() {
            let layout = Layout::left_to_right(Align::Center);
            area(ui, vec2(end - start, height), layout, |ui| add(ui, column));
        }
        let amount = vec2(end - CELLS[CELLS.len() - 1].1, height);
        area(ui, amount, Layout::right_to_left(Align::Center), |ui| {
            add(ui, CELLS.len());
        });
    });
}

/// The progress bar: egui's progress bar, its track in the faint background color.
fn progress(ui: &mut Ui, percent: u32) {
    ui.add(
        ProgressBar::new(percent as f32 / 100.0)
            .desired_width(spec::PROGRESS_WIDTH)
            .desired_height(spec::PROGRESS_HEIGHT)
            .corner_radius(spec::PROGRESS_HEIGHT / 2.0)
            .fill(color(spec::PRIMARY)),
    );
}

/// Sets the color of the 1 px separators under the header or the rows.
fn set_rule(ui: &mut Ui, rule: u32) {
    ui.visuals_mut().widgets.noninteractive.bg_stroke = Stroke::new(1.0, color(rule));
}
