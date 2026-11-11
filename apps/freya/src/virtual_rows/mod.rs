//! The virtual scenario: the rows of the rows scenario in freya's `VirtualScrollView`, which
//! builds only the rows in the viewport. Each row stops the wheel input with the rows scenario's
//! handler, so the content moves 1:1 with the touchpad as there.

use freya::prelude::*;

use crate::page;
use crate::rows::row::Row;
use crate::rows::wheel::WheelScroll;
use crate::spec;

/// Opens the virtual window and runs the event loop until it closes.
pub fn run(title: &'static str, count: usize) {
    page::launch_app(title, VirtualRows { count });
}

/// The root component: the virtual scroll view of `count` rows.
struct VirtualRows {
    count: usize,
}

impl App for VirtualRows {
    fn render(&self) -> impl IntoElement {
        let wheel = WheelScroll::use_new();
        rect()
            .expanded()
            .background(page::color(spec::BACKGROUND))
            .font_family(spec::FONT_FAMILY)
            .font_size(spec::FONT_SIZE)
            .child(
                VirtualScrollView::new_controlled(
                    move |item: VirtualItem, _: &()| {
                        rect()
                            .key(item.index)
                            .width(Size::fill())
                            .on_wheel(move |event| wheel.apply(event))
                            .child(Row::new(item.index))
                            .into()
                    },
                    wheel.controller(),
                )
                .auto_hide_scrollbars(false)
                .length(self.count)
                .item_size(spec::ROW_HEIGHT),
            )
    }
}
