//! The rows view: a full-window vertical scroll container that holds every row as a real
//! element, and scrolls 1:1 with the input in logical pixels.

use gpui_kit::*;

const ROW_HEIGHT: f32 = 32.0;
const PADDING_X: f32 = 12.0;
const FONT_SIZE: f32 = 14.0;

const BACKGROUND: u32 = 0xffffff;
const ROW_EVEN: u32 = 0xffffff;
const ROW_ODD: u32 = 0xf6f8fa;
const ROW_HOVER: u32 = 0xcfe3ff;
const ROW_BORDER: u32 = 0xe5e7eb;
const LABEL_COLOR: u32 = 0x1f2328;
const VALUE_COLOR: u32 = 0x6e7781;

/// The Wayland backend multiplies touchpad axis distances by this factor.
const PLATFORM_PIXEL_SCALE: f32 = 3.0;
/// The Wayland backend reports one wheel detent as this many lines.
const LINES_PER_DETENT: f32 = 3.0;
/// The axis distance of one wheel detent in logical pixels.
const DETENT_PIXELS: f32 = 15.0;

pub struct Rows {
    count: usize,
    scroll: ScrollHandle,
}

impl Rows {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            scroll: ScrollHandle::new(),
        }
    }

    /// Moves the content by the axis distance of the event and stops the built-in scroll
    /// handler, which applies the platform line and pixel factors.
    fn on_scroll(&mut self, event: &ScrollEvent, _: &mut Window, cx: &mut Context<Self>) {
        let delta = match event.delta {
            ScrollDelta::Pixels(delta) => delta.y / PLATFORM_PIXEL_SCALE,
            ScrollDelta::Lines(lines) => px(lines.y / LINES_PER_DETENT * DETENT_PIXELS),
        };
        cx.stop_propagation();
        let mut offset = self.scroll.offset();
        let max = self.scroll.max_offset();
        let y = (offset.y + delta).clamp(-max.y, px(0.0));
        if y != offset.y {
            offset.y = y;
            self.scroll.set_offset(offset);
            cx.notify();
        }
    }
}

impl Render for Rows {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("scroll")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .bg(rgb(BACKGROUND))
            .child(
                div()
                    .id("rows")
                    .flex()
                    .flex_col()
                    .w_full()
                    .on_scroll(cx.listener(Self::on_scroll))
                    .children((0..self.count).map(row)),
            )
    }
}

fn row(i: usize) -> impl IntoElement {
    let background = if i.is_multiple_of(2) { ROW_EVEN } else { ROW_ODD };
    div()
        .id(i)
        .flex()
        .flex_row()
        .flex_none()
        .items_center()
        .justify_between()
        .w_full()
        .h(px(ROW_HEIGHT))
        .pl(px(PADDING_X))
        .pr(px(PADDING_X))
        .border_b_1()
        .border_color(rgb(ROW_BORDER))
        .bg(rgb(background))
        .hover(|style| style.bg(rgb(ROW_HOVER)))
        .text_size(px(FONT_SIZE))
        .child(
            div()
                .text_color(rgb(LABEL_COLOR))
                .child(format!("Row {i:05}")),
        )
        .child(
            div()
                .text_color(rgb(VALUE_COLOR))
                .child(format!("{}", (i * 7919) % 10000)),
        )
}
