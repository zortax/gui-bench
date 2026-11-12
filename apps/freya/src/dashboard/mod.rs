//! The dashboard scenario: the analytics page of `apps/shared/dashboard.rs` with a sidebar, a top
//! bar, and a scroll view with metric cards, a chart, filters and a table of orders.
//!
//! The root component owns the state as `State`s (the nav item, the search text, the chart
//! series, the compare switch and the status filter) and only hands them down, so it renders
//! once; each part is a component that reads the states it shows and renders alone when they
//! change.

mod chart;
mod filters;
mod kpis;
mod orders;
mod sidebar;
#[path = "../../../shared/dashboard.rs"]
mod spec;
mod topbar;

use std::borrow::Cow;
use std::rc::Rc;

use freya::prelude::*;

use crate::page::{self, color};
use chart::Chart;
use filters::Filters;
use kpis::Kpis;
use orders::Orders;
use sidebar::Sidebar;
use topbar::TopBar;

/// Opens the dashboard window with `count` orders and runs the event loop until it closes.
pub fn run(title: &'static str, count: usize) {
    page::launch_app(title, Dashboard { count });
}

/// The root: the sidebar beside the top bar and the scrolling content.
struct Dashboard {
    count: usize,
}

impl App for Dashboard {
    fn render(&self) -> impl IntoElement {
        let count = self.count;
        let orders = use_hook(|| Rc::new(spec::orders(count).into_iter().map(Rc::new).collect()));
        let active = use_state(|| 0usize);
        let search = use_state(String::new);
        let series = use_state(|| 0usize);
        let compare = use_state(|| false);
        let shown = use_state(|| [true; 4]);

        let content = rect()
            .width(Size::fill())
            .padding(spec::CONTENT_PADDING)
            .spacing(spec::GAP)
            .child(Kpis)
            .child(
                rect()
                    .horizontal()
                    .content(Content::Flex)
                    .width(Size::fill())
                    .height(Size::px(spec::ROW2_HEIGHT))
                    .spacing(spec::GAP)
                    .child(Chart { series, compare })
                    .child(Filters {
                        shown,
                        compare,
                        series,
                        search,
                    }),
            )
            .child(Orders {
                orders,
                shown,
                search,
            });

        rect()
            .expanded()
            .horizontal()
            .background(color(spec::PAGE_BG))
            .font_family(crate::spec::FONT_FAMILY)
            .font_size(spec::FONT_SIZE)
            .color(color(spec::TEXT))
            .child(Sidebar { active })
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    .child(TopBar { search })
                    .child(ScrollView::new().child(content)),
            )
    }
}

/// Returns a one-line label of `value` in `size` px, `weight` and `fill`.
fn text(value: impl Into<Cow<'static, str>>, size: f32, weight: FontWeight, fill: u32) -> Label {
    label()
        .text(value)
        .max_lines(1)
        .font_size(size)
        .font_weight(weight)
        .color(color(fill))
}

/// Returns the freya shadow of a spec shadow.
fn shadow(shadow: spec::Shadow) -> Shadow {
    let (r, g, b) = crate::spec::rgb(shadow.color);
    Shadow::new()
        .x(shadow.x)
        .y(shadow.y)
        .blur(shadow.blur)
        .color(Color::from_af32rgb(shadow.alpha, r, g, b))
}

/// Returns a white card with rounded corners and the card shadow.
fn card() -> Rect {
    rect()
        .background(color(spec::CARD_BG))
        .corner_radius(spec::CARD_RADIUS)
        .shadow(shadow(spec::SHADOW_CARD))
}

/// Returns a pill of `height` with the label `value` in 12 px semibold.
fn pill(value: impl Into<Cow<'static, str>>, height: f32, pad_x: f32, colors: (u32, u32)) -> Rect {
    rect()
        .height(Size::px(height))
        .padding(Gaps::new(0.0, pad_x, 0.0, pad_x))
        .corner_radius(height / 2.0)
        .background(color(colors.0))
        .center()
        .child(text(value, 12.0, FontWeight::SEMI_BOLD, colors.1))
}

/// Returns a circle of `size` in `fill` with the initials in `font` px semibold white.
fn avatar(initials: impl Into<Cow<'static, str>>, size: f32, fill: u32, font: f32) -> Rect {
    rect()
        .width(Size::px(size))
        .height(Size::px(size))
        .corner_radius(size / 2.0)
        .background(color(fill))
        .center()
        .child(text(initials, font, FontWeight::SEMI_BOLD, 0xffffff))
}

/// The theme of the primary buttons: `PRIMARY` with white text.
fn primary_colors() -> ButtonColorsThemePartial {
    ButtonColorsThemePartial::new()
        .background(color(spec::PRIMARY))
        .hover_background(color(0x4338ca))
        .border_fill(Color::TRANSPARENT)
        .focus_border_fill(color(0x818cf8))
        .color(color(spec::PRIMARY_TEXT))
}

/// The layout of a button of `width`, `CONTROL_HEIGHT` tall, with radius 8.
fn button_layout(width: f32) -> ButtonLayoutThemePartial {
    ButtonLayoutThemePartial::new()
        .width(Size::px(width))
        .height(Size::px(spec::CONTROL_HEIGHT))
        .padding(Gaps::new_all(0.0))
        .margin(Gaps::new_all(0.0))
        .corner_radius(8.0)
}

/// Returns a primary button of `width` with `value`, on a rect that carries the small shadow
/// (freya's button has no shadow in its theme).
fn primary_button(value: &'static str, width: f32) -> Rect {
    rect()
        .corner_radius(8.0)
        .shadow(shadow(spec::SHADOW_SMALL))
        .child(
            Button::new()
                .filled()
                .theme_colors(primary_colors())
                .theme_layout(button_layout(width))
                .child(text(
                    value,
                    spec::FONT_SIZE,
                    FontWeight::SEMI_BOLD,
                    spec::PRIMARY_TEXT,
                )),
        )
}
