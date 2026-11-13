//! The top bar: the title of the page on the left; freya's `Input` for the search, the live
//! switch with its spinner, the export menu and the avatar on the right.

use freya::prelude::*;

use super::live::LiveSpinner;
use super::menu::ExportMenu;
use super::{avatar, bottom_border, color, spec, switch, text};

/// The top bar. It reads the active nav item for its title. The input reads and writes the
/// search state itself, and the switch and the spinner the live state, so typing and the live
/// switch do not render the bar.
#[derive(PartialEq)]
pub struct TopBar {
    pub active: State<usize>,
    pub search: State<String>,
    pub live: State<bool>,
}

impl Component for TopBar {
    fn render(&self) -> impl IntoElement {
        let input = Input::new(self.search)
            .placeholder(spec::SEARCH_PLACEHOLDER)
            .width(Size::px(spec::SEARCH_WIDTH))
            .height(Size::px(spec::CONTROL_HEIGHT))
            .theme_colors(
                InputColorsThemePartial::new()
                    .background(color(spec::INPUT_BG))
                    .focus_background(color(spec::CARD_BG))
                    .border_fill(color(spec::INPUT_BORDER))
                    .focus_border_fill(color(spec::PRIMARY))
                    .color(color(spec::TEXT))
                    .placeholder_color(color(spec::SIDEBAR_MUTED)),
            )
            .theme_layout(
                InputLayoutThemePartial::new()
                    .corner_radius(8.0)
                    .padding(Gaps::new(0.0, 12.0, 0.0, 12.0)),
            );
        let search_end = spec::SEARCH_X + spec::SEARCH_WIDTH;
        let switch_end = spec::LIVE_X + spec::SWITCH_WIDTH;
        let spinner_end = spec::SPINNER_X + spec::SPINNER_SIZE;
        let export_end = spec::EXPORT_X + spec::EXPORT_WIDTH;

        rect()
            .horizontal()
            .width(Size::fill())
            .height(Size::px(spec::TOPBAR_HEIGHT))
            .padding(Gaps::new(
                0.0,
                spec::CONTENT_PADDING,
                0.0,
                spec::TOPBAR_TITLE_X - spec::SIDEBAR_WIDTH,
            ))
            .main_align(Alignment::SpaceBetween)
            .cross_align(Alignment::Center)
            .background(color(spec::TOPBAR_BG))
            .border(bottom_border(spec::RULE))
            .child(text(
                spec::NAV_ITEMS[*self.active.read()],
                spec::TITLE_FONT + 4.0,
                FontWeight::SEMI_BOLD,
                spec::TEXT,
            ))
            .child(
                rect()
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .child(input)
                    .child(gap(spec::LIVE_X - search_end))
                    .child(switch(self.live))
                    .child(gap(spec::SPINNER_X - switch_end))
                    .child(LiveSpinner { live: self.live })
                    .child(gap(spec::EXPORT_X - spinner_end))
                    .child(ExportMenu)
                    .child(gap(spec::TOPBAR_AVATAR_X - export_end))
                    .child(avatar(
                        spec::USER_INITIALS,
                        spec::CONTROL_HEIGHT,
                        spec::USER_COLOR,
                        13.0,
                    )),
            )
    }
}

/// Returns the empty space of `width` between two controls.
fn gap(width: f32) -> Rect {
    rect().width(Size::px(width))
}
