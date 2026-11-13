//! The app scenario: a sidebar with nav, a top bar with the search, the live switch and the
//! export menu, and one scroll view with the page of the active nav item: the overview (the
//! metric cards, the chart, the filters and the orders table), the messages feed, or an empty
//! page (`apps/shared/dashboard.rs`). The list scenario (`list.rs`) shows the orders alone.
//!
//! The root component owns the state as `State`s (the nav item, the search text, the chart
//! series, the compare switch, the status filter, the sorting and the live switch with its tick)
//! and only hands them down, so it renders once; each part is a component that reads the states
//! it shows and renders alone when they change.

mod chart;
mod content;
mod filters;
mod kpis;
mod list;
mod live;
mod menu;
mod messages;
mod orders;
mod sidebar;
#[path = "../../../shared/dashboard.rs"]
mod spec;
mod topbar;

use std::borrow::Cow;
use std::rc::Rc;

use freya::prelude::*;

use content::PageView;
use list::OrderList;
use sidebar::Sidebar;
use topbar::TopBar;

const APP_ID: &str = "gui-bench.freya";
/// The font family of all text.
const FONT_FAMILY: &str = "sans-serif";

/// Opens the app window with `count` orders and messages and runs the event loop until it
/// closes.
pub fn run_app(title: &'static str, count: usize) {
    launch_app(title, Dashboard { count });
}

/// Opens the list window with `count` order rows and runs the event loop until it closes.
pub fn run_list(title: &'static str, count: usize) {
    launch_app(title, OrderList { count });
}

/// Opens an undecorated window of the spec size with `app` as its root and runs the event loop
/// until it closes.
fn launch_app(title: &'static str, app: impl App) {
    let window = WindowConfig::new_app(app)
        .with_title(title)
        .with_app_id(APP_ID)
        .with_size(f64::from(spec::WINDOW.0), f64::from(spec::WINDOW.1))
        .with_decorations(false)
        .with_background(color(spec::PAGE_BG));
    launch(LaunchConfig::new().with_window(window));
}

/// The root: the sidebar beside the top bar and the scrolling content.
struct Dashboard {
    count: usize,
}

impl App for Dashboard {
    fn render(&self) -> impl IntoElement {
        let count = self.count;
        let orders = use_hook(|| Rc::new(spec::orders(count)));
        let active = use_state(|| 0usize);
        let search = use_state(String::new);
        let series = use_state(|| 0usize);
        let compare = use_state(|| false);
        let shown = use_state(|| [true; 4]);
        let sorted = use_state(|| false);
        let live = use_state(|| false);
        let tick = live::use_live(live);

        root()
            .horizontal()
            .background(color(spec::PAGE_BG))
            .child(Sidebar { active })
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    .child(TopBar {
                        active,
                        search,
                        live,
                    })
                    .child(ScrollView::new().child(PageView {
                        count,
                        orders,
                        active,
                        tick,
                        search,
                        series,
                        compare,
                        shown,
                        sorted,
                    })),
            )
    }
}

/// Returns the rect that fills the window and sets the font and the text color of the spec.
fn root() -> Rect {
    rect()
        .expanded()
        .font_family(FONT_FAMILY)
        .font_size(spec::FONT_SIZE)
        .color(color(spec::TEXT))
}

/// Returns the freya color of a spec color.
const fn color(hex: u32) -> Color {
    Color::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// Returns a 1 px border of `fill` on the bottom edge only, inside the element.
fn bottom_border(fill: u32) -> Border {
    Border::new()
        .fill(color(fill))
        .width(BorderWidth {
            top: 0.0,
            right: 0.0,
            bottom: 1.0,
            left: 0.0,
        })
        .alignment(BorderAlignment::Inner)
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
    Shadow::new()
        .x(shadow.x)
        .y(shadow.y)
        .blur(shadow.blur)
        .color(Color::from_af32rgb(
            shadow.alpha,
            (shadow.color >> 16) as u8,
            (shadow.color >> 8) as u8,
            shadow.color as u8,
        ))
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

/// Returns a primary button of `width` with `value`.
fn primary_button(value: &'static str, width: f32) -> Button {
    Button::new()
        .filled()
        .theme_colors(primary_colors())
        .theme_layout(button_layout(width))
        .child(text(
            value,
            spec::FONT_SIZE,
            FontWeight::SEMI_BOLD,
            spec::PRIMARY_TEXT,
        ))
}

/// Returns `button` on a rect that carries the small shadow (freya's button has no shadow in its
/// theme).
fn raised(button: Button) -> Rect {
    rect()
        .corner_radius(8.0)
        .shadow(shadow(spec::SHADOW_SMALL))
        .child(button)
}

/// Returns freya's `Switch` for `value` with the look of the spec: 40x22 with an 18 px knob.
fn switch(value: State<bool>) -> Switch {
    Switch::new(value)
        .theme_colors(
            SwitchColorsThemePartial::new()
                .background(color(spec::SWITCH_OFF))
                .thumb_background(color(spec::CARD_BG))
                .toggled_background(color(spec::PRIMARY))
                .toggled_thumb_background(color(spec::CARD_BG))
                .focus_border_fill(color(spec::SPARK)),
        )
        .theme_layout(
            SwitchLayoutThemePartial::new()
                .margin(Gaps::new_all(0.0))
                .width(spec::SWITCH_WIDTH)
                .height(spec::SWITCH_HEIGHT)
                .padding(2.0)
                .thumb_size(18.0)
                .toggled_thumb_size(18.0)
                .pressed_thumb_size_offset(0.0)
                .thumb_offset(0.0)
                .toggled_thumb_offset(spec::SWITCH_WIDTH - 18.0 - 4.0),
        )
}
