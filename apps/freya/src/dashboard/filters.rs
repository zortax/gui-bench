//! The filter card: freya's `Checkbox` per status, its `Switch` for the compare option and its
//! `Button`s for Reset and Apply.

use freya::prelude::*;

use super::{button_layout, card, color, primary_button, raised, spec, switch, text};

/// The filter card. It reads the status filter; the switch reads the compare state itself.
#[derive(PartialEq)]
pub struct Filters {
    pub shown: State<[bool; 4]>,
    pub compare: State<bool>,
    pub series: State<usize>,
    pub search: State<String>,
    pub sorted: State<bool>,
}

impl Component for Filters {
    fn render(&self) -> impl IntoElement {
        let mut shown = self.shown;
        let mut compare = self.compare;
        let mut series = self.series;
        let mut search = self.search;
        let mut sorted = self.sorted;
        let checked = *shown.read();
        let checks = spec::Status::ALL.iter().enumerate().map(|(k, status)| {
            rect()
                .horizontal()
                .width(Size::fill())
                .height(Size::px(spec::CHECK_STRIDE))
                .spacing(10.0)
                .cross_align(Alignment::Center)
                .on_press(move |_| shown.write()[k] ^= true)
                .child(
                    Checkbox::new()
                        .selected(checked[k])
                        .size(spec::CHECK_SIZE)
                        .theme(
                            CheckboxThemePartial::new()
                                .selected_fill(color(spec::PRIMARY))
                                .unselected_fill(color(spec::INPUT_BORDER))
                                .selected_icon_fill(color(spec::PRIMARY_TEXT))
                                .border_fill(color(spec::SPARK)),
                        ),
                )
                .child(text(
                    status.label(),
                    spec::FONT_SIZE,
                    FontWeight::NORMAL,
                    spec::TEXT,
                ))
        });
        let reset = Button::new()
            .theme_colors(
                ButtonColorsThemePartial::new()
                    .background(color(spec::SECONDARY_BG))
                    .hover_background(color(spec::INPUT_BG))
                    .border_fill(color(spec::SECONDARY_BORDER))
                    .focus_border_fill(color(spec::PRIMARY))
                    .color(color(spec::TEXT)),
            )
            .theme_layout(button_layout(spec::BUTTON_WIDTH))
            .on_press(move |_| {
                shown.set([true; 4]);
                compare.set(false);
                series.set(0);
                search.set(String::new());
                sorted.set(false);
            })
            .child(text(
                spec::RESET_LABEL,
                spec::FONT_SIZE,
                FontWeight::SEMI_BOLD,
                spec::TEXT,
            ));

        // The checkboxes from y 56 in the card, the compare row at 196 and the buttons at 264.
        card()
            .width(Size::px(spec::FILTERS_WIDTH))
            .height(Size::fill())
            .padding(spec::CARD_PADDING)
            .child(rect().height(Size::px(36.0)).child(text(
                spec::FILTERS_TITLE,
                spec::TITLE_FONT,
                FontWeight::SEMI_BOLD,
                spec::TEXT,
            )))
            .children(checks)
            .child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .height(Size::px(28.0))
                    .margin(Gaps::new(12.0, 0.0, 0.0, 0.0))
                    .main_align(Alignment::SpaceBetween)
                    .cross_align(Alignment::Center)
                    .child(text(
                        spec::COMPARE_LABEL,
                        spec::FONT_SIZE,
                        FontWeight::NORMAL,
                        spec::TEXT,
                    ))
                    .child(switch(compare)),
            )
            .child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .margin(Gaps::new(40.0, 0.0, 0.0, 0.0))
                    .main_align(Alignment::End)
                    .spacing(spec::APPLY_X - spec::RESET_X - spec::BUTTON_WIDTH)
                    .child(reset)
                    .child(raised(primary_button(
                        spec::APPLY_LABEL,
                        spec::BUTTON_WIDTH,
                    ))),
            )
    }
}
