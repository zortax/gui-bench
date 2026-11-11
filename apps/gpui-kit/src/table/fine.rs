//! The fine variant: every row is an entity that holds its own label, value and selected state,
//! and the table embeds each one as a cached view, so a render of the table reuses every row
//! that was not notified. An edit or a tick updates and notifies only the rows it changes; a
//! structural operation reorders the row entities, which keep their ids; a selection change
//! notifies the old and the new selected row.

use std::mem;

use gpui_kit::*;

use super::{keys, ticker};
use crate::cli::Ticker;
use crate::page::{header, list, page, plain_row};
use crate::spec::{self, Ids, Op, Row, TableRow};

/// One row: its data and whether it is selected.
struct RowView {
    id: u64,
    label: SharedString,
    value: u32,
    selected: bool,
}

impl RowView {
    fn new(row: Row) -> Self {
        Self {
            id: row.id,
            label: row.label.into(),
            value: row.value,
            selected: false,
        }
    }
}

impl Render for RowView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        plain_row(self.label.clone(), self.value, self.selected)
    }
}

/// A row while an operation runs: a row entity, marked when the operation toggled its label, or
/// the data of a new row whose entity the table makes afterwards.
enum Slot {
    Kept {
        view: Entity<RowView>,
        toggled: bool,
    },
    New(Row),
}

impl TableRow for Slot {
    fn from_row(row: Row) -> Self {
        Self::New(row)
    }

    fn toggle_suffix(&mut self) {
        match self {
            Self::Kept { toggled, .. } => *toggled = !*toggled,
            Self::New(row) => row.toggle_suffix(),
        }
    }
}

/// The table with one entity per row.
pub struct FineTable {
    count: usize,
    ids: Ids,
    rows: Vec<Entity<RowView>>,
    selected: Option<u64>,
    ticker: Option<Ticker>,
    tick: u64,
    focus: FocusHandle,
    _timer: Option<Task<()>>,
}

impl FineTable {
    /// Returns the table with `count` rows and focuses it, so it hears the function keys.
    pub fn new(
        count: usize,
        ticker: Option<Ticker>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut ids = Ids::default();
        let rows = ids
            .rows(count)
            .into_iter()
            .map(|row| cx.new(|_| RowView::new(row)))
            .collect();
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        Self {
            count,
            ids,
            rows,
            selected: None,
            ticker,
            tick: 0,
            focus,
            _timer: ticker.map(|t| ticker::every(t.interval_ms, cx, Self::tick)),
        }
    }

    /// Runs the operation on the row entities. Rows whose label it toggles are notified; new
    /// rows get new entities; the table is notified only when the list changes.
    fn run(&mut self, op: Op, cx: &mut Context<Self>) {
        let mut slots: Vec<Slot> = mem::take(&mut self.rows)
            .into_iter()
            .map(|view| Slot::Kept {
                view,
                toggled: false,
            })
            .collect();
        op.apply(&mut slots, &mut self.ids, self.count);
        self.rows = slots
            .into_iter()
            .map(|slot| match slot {
                Slot::Kept { view, toggled } => {
                    if toggled {
                        view.update(cx, |row, cx| {
                            row.label = spec::toggle_suffix(&row.label).into();
                            cx.notify();
                        });
                    }
                    view
                }
                Slot::New(row) => cx.new(|_| RowView::new(row)),
            })
            .collect();
        if op.changes_structure() {
            cx.notify();
        }
    }

    /// Moves the selection and notifies the row that loses it and the row that gets it.
    fn select(&mut self, id: u64, cx: &mut Context<Self>) {
        let old = self.selected;
        let new = super::toggle(old, id);
        self.selected = new;
        for (id, selected) in [(old, false), (new, true)] {
            let Some(id) = id else { continue };
            if let Some(view) = self.rows.iter().find(|view| view.read(cx).id == id) {
                view.update(cx, |row, cx| {
                    row.selected = selected;
                    cx.notify();
                });
            }
        }
    }

    /// Writes the tick values into the ticked rows and notifies them, and the table for its
    /// header.
    fn tick(&mut self, cx: &mut Context<Self>) {
        let Some(ticker) = self.ticker else { return };
        self.tick += 1;
        let tick = self.tick;
        for at in spec::ticker_positions(self.rows.len(), ticker.updates) {
            self.rows[at].update(cx, |row, cx| {
                row.value = spec::ticker_value(row.id, tick);
                cx.notify();
            });
        }
        cx.notify();
    }
}

impl Render for FineTable {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = super::status(self.rows.len(), self.ticker, self.tick);
        let root = keys::on_ops(page().track_focus(&self.focus), cx, Self::run);
        let rows = self.rows.iter().map(|view| {
            let id = view.read(cx).id;
            div()
                .id(id)
                .flex_none()
                .w_full()
                .h(px(spec::ROW_HEIGHT))
                .on_click(cx.listener(move |this, _, _, cx| this.select(id, cx)))
                .child(view.clone().cached(StyleRefinement::default().size_full()))
        });
        root.child(header(status)).child(list().children(rows))
    }
}
