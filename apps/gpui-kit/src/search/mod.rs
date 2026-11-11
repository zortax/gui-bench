//! The search and typing scenarios: a header with gpui-component's text input, focused at
//! startup, above `count` rows. With `filter`, the view hears every change of the input and
//! shows the rows whose label holds the text, keyed by id; without, the rows stay.

use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::*;

use crate::page::{header, list, page, plain_row};
use crate::spec::{self, Ids, Row};

/// The page with the input and the rows.
pub struct Search {
    rows: Vec<Row>,
    query: SharedString,
    input: Entity<InputState>,
    _changes: Option<Subscription>,
}

impl Search {
    pub fn new(count: usize, filter: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Search"));
        input.update(cx, |input, cx| input.focus(window, cx));
        let changes = filter.then(|| {
            cx.subscribe_in(&input, window, |this, input, event: &InputEvent, _, cx| {
                if let InputEvent::Change = event {
                    this.query = input.read(cx).value();
                    cx.notify();
                }
            })
        });
        Self {
            rows: Ids::default().rows(count),
            query: SharedString::default(),
            input,
            _changes: changes,
        }
    }
}

impl Render for Search {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let rows = self
            .rows
            .iter()
            .filter(|row| spec::matches(&row.label, &self.query))
            .map(|row| plain_row(row.label.clone(), row.value, false).id(row.id));
        let field = div()
            .flex_none()
            .w(px(spec::INPUT_WIDTH))
            .child(Input::new(&self.input));
        page()
            .child(header(spec::status(self.rows.len())).child(field))
            .child(list().children(rows))
    }
}
