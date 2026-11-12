//! The top bar: the page title on the left; freya's `Input` for the search, the export button
//! and the avatar on the right.

use freya::prelude::*;

use super::{avatar, primary_button, spec, text};
use crate::page::{bottom_border, color};

/// The top bar. The input reads and writes the search state itself, so typing renders the input
/// and the orders, not the bar.
#[derive(PartialEq)]
pub struct TopBar {
    pub search: State<String>,
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
        let end = spec::WINDOW.0 - spec::CONTENT_PADDING;
        let avatar_gap = end - spec::CONTROL_HEIGHT - (spec::EXPORT_X + spec::EXPORT_WIDTH);
        let export_gap = spec::EXPORT_X - (spec::SEARCH_X + spec::SEARCH_WIDTH);

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
                spec::PAGE_TITLE,
                spec::TITLE_FONT + 4.0,
                FontWeight::SEMI_BOLD,
                spec::TEXT,
            ))
            .child(
                rect()
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .child(input)
                    .child(
                        rect()
                            .margin(Gaps::new(0.0, avatar_gap, 0.0, export_gap))
                            .child(primary_button(spec::EXPORT_LABEL, spec::EXPORT_WIDTH)),
                    )
                    .child(avatar(
                        spec::USER_INITIALS,
                        spec::CONTROL_HEIGHT,
                        spec::USER_COLOR,
                        13.0,
                    )),
            )
    }
}
