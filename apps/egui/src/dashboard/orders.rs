//! The orders card: a header of column titles, then one row per order that passes the filters
//! and the search, all of them laid out every pass (no virtualization). A row is built like the
//! rows of the rows scenario: a frame of fixed-width cells, its fill taken from its response, and
//! a separator under it.

use eframe::egui::{Align, CornerRadius, Frame, Layout, ProgressBar, Separator, Stroke, Ui, vec2};

use super::spec::{self, Order};
use super::style::{Weight, area, avatar, card, color, pill, text};

/// The cells of a row from the left of the card: each from its column's left edge to the next
/// one; the amount, right-aligned, from the end of the progress bar to its own end.
const CELLS: [(f32, f32); 5] = [
    (spec::COL_ORDER, spec::COL_CUSTOMER),
    (spec::COL_CUSTOMER, spec::COL_STATUS),
    (spec::COL_STATUS, spec::COL_PROGRESS),
    (
        spec::COL_PROGRESS,
        spec::COL_PROGRESS + spec::PROGRESS_WIDTH,
    ),
    (
        spec::COL_PROGRESS + spec::PROGRESS_WIDTH,
        spec::COL_AMOUNT_END,
    ),
];
/// The space between the avatar and the name.
const NAME_GAP: f32 = 12.0;

/// Shows the orders card with the `orders` that pass `shown` and `search`.
pub fn show(ui: &mut Ui, orders: &[Order], shown: &[bool; 4], search: &str) {
    card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        // The progress track color, set once for every row.
        ui.visuals_mut().extreme_bg_color = color(spec::TRACK);
        header(ui);
        set_rule(ui, spec::ROW_RULE);
        let mut visible = orders
            .iter()
            .filter(|order| spec::visible(order, shown, search))
            .peekable();
        while let Some(order) = visible.next() {
            row(ui, order, visible.peek().is_none());
        }
    });
}

/// The header: the column titles on the header fill with the card's top corners, and the rule
/// under it.
fn header(ui: &mut Ui) {
    let radius = spec::CARD_RADIUS as u8;
    Frame::new()
        .fill(color(spec::TABLE_HEADER_BG))
        .corner_radius(CornerRadius {
            nw: radius,
            ne: radius,
            sw: 0,
            se: 0,
        })
        .show(ui, |ui| {
            cells(ui, spec::TABLE_HEADER_HEIGHT - 1.0, |ui, column| {
                let title = text(spec::COLUMNS[column], 12.0, Weight::Semibold, spec::MUTED);
                ui.label(title);
            });
        });
    set_rule(ui, spec::RULE);
    ui.add(Separator::default().spacing(1.0));
}

/// One order row; the `last` one takes the card's bottom corners for its hover fill.
fn row(ui: &mut Ui, order: &Order, last: bool) {
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

/// Lays out the cells of a row `height` tall; `add` adds the content of cell `column`.
fn cells(ui: &mut Ui, height: f32, mut add: impl FnMut(&mut Ui, usize)) {
    let size = vec2(ui.available_width(), height);
    area(ui, size, Layout::left_to_right(Align::Center), |ui| {
        ui.add_space(CELLS[0].0);
        for (column, (start, end)) in CELLS.into_iter().enumerate() {
            let layout = if column == CELLS.len() - 1 {
                Layout::right_to_left(Align::Center)
            } else {
                Layout::left_to_right(Align::Center)
            };
            area(ui, vec2(end - start, height), layout, |ui| add(ui, column));
        }
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
