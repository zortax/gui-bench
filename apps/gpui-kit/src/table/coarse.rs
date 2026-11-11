//! The coarse variant: one view holds every row and the selection, and any change, an edit of
//! one row or a new selection included, notifies the view, which renders every row anew.

use gpui_kit::*;

use super::{keys, ticker};
use crate::cli::Ticker;
use crate::page::{header, list, page, plain_row};
use crate::spec::{self, Ids, Op, Row};

/// The table with one state for all rows.
pub struct CoarseTable {
    count: usize,
    ids: Ids,
    rows: Vec<Row>,
    selected: Option<u64>,
    ticker: Option<Ticker>,
    tick: u64,
    focus: FocusHandle,
    _timer: Option<Task<()>>,
}

impl CoarseTable {
    /// Returns the table with `count` rows and focuses it, so it hears the function keys.
    pub fn new(
        count: usize,
        ticker: Option<Ticker>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut ids = Ids::default();
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        Self {
            count,
            rows: ids.rows(count),
            ids,
            selected: None,
            ticker,
            tick: 0,
            focus,
            _timer: ticker.map(|t| ticker::every(t.interval_ms, cx, Self::tick)),
        }
    }

    fn run(&mut self, op: Op, cx: &mut Context<Self>) {
        op.apply(&mut self.rows, &mut self.ids, self.count);
        cx.notify();
    }

    fn select(&mut self, id: u64, cx: &mut Context<Self>) {
        self.selected = super::toggle(self.selected, id);
        cx.notify();
    }

    fn tick(&mut self, cx: &mut Context<Self>) {
        let Some(ticker) = self.ticker else { return };
        self.tick += 1;
        for at in spec::ticker_positions(self.rows.len(), ticker.updates) {
            let row = &mut self.rows[at];
            row.value = spec::ticker_value(row.id, self.tick);
        }
        cx.notify();
    }
}

impl Render for CoarseTable {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = super::status(self.rows.len(), self.ticker, self.tick);
        let root = keys::on_ops(page().track_focus(&self.focus), cx, Self::run);
        let rows = self.rows.iter().map(|row| {
            let id = row.id;
            plain_row(row.label.clone(), row.value, self.selected == Some(id))
                .id(id)
                .on_click(cx.listener(move |this, _, _, cx| this.select(id, cx)))
        });
        root.child(header(status)).child(list().children(rows))
    }
}
