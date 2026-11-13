//! The four metric cards: a label, the value, the delta pill and a sparkline of plain bars.

use freya::prelude::*;

use super::{card, color, pill, spec, text};

/// The row of metric cards. It only hands the live tick down, so it renders once.
#[derive(PartialEq)]
pub struct Kpis {
    pub tick: State<u64>,
}

impl Component for Kpis {
    fn render(&self) -> impl IntoElement {
        rect()
            .horizontal()
            .content(Content::Flex)
            .width(Size::fill())
            .height(Size::px(spec::KPI_HEIGHT))
            .spacing(spec::GAP)
            .children((0..spec::KPI_COUNT).map(|index| {
                KpiCard {
                    index,
                    tick: self.tick,
                }
                .into_element()
            }))
    }
}

/// Metric card `index`, which shares the width of the row with the others. It reads the live
/// tick, so a tick renders the card with its next value.
#[derive(PartialEq)]
struct KpiCard {
    index: usize,
    tick: State<u64>,
}

impl Component for KpiCard {
    fn render(&self) -> impl IntoElement {
        let kpi = &spec::KPIS[self.index];
        let value = spec::kpi_value(self.index, *self.tick.read());
        let colors = if kpi.up {
            (spec::UP_BG, spec::UP_TEXT)
        } else {
            (spec::DOWN_BG, spec::DOWN_TEXT)
        };
        let bars = kpi.spark.iter().map(|&height| {
            rect()
                .width(Size::px(spec::SPARK_BAR_WIDTH))
                .height(Size::px(height))
                .corner_radius(2.0)
                .background(color(spec::SPARK))
        });

        // The label at y 18, the value at y 40 and the pill at y 78, each in a box of its own.
        card()
            .width(Size::flex(1.0))
            .height(Size::fill())
            .padding(Gaps::new(18.0, spec::CARD_PADDING, 0.0, spec::CARD_PADDING))
            .child(rect().height(Size::px(22.0)).child(text(
                kpi.label,
                13.0,
                FontWeight::NORMAL,
                spec::MUTED,
            )))
            .child(rect().height(Size::px(38.0)).child(text(
                value,
                24.0,
                FontWeight::BOLD,
                spec::TEXT,
            )))
            .child(pill(kpi.delta, 22.0, 8.0, colors))
            // The sparkline sits on the inner edges of the padding: its right end 20 px inside
            // the card (the right padding), its bottom on card y 92.
            .child(
                rect()
                    .position(
                        Position::new_absolute()
                            .right(0.0)
                            .bottom(spec::KPI_HEIGHT - 92.0),
                    )
                    .horizontal()
                    .spacing(spec::SPARK_STEP - spec::SPARK_BAR_WIDTH)
                    .cross_align(Alignment::End)
                    .children(bars),
            )
    }
}
