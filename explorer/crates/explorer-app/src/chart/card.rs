//! The card around a chart: a bold title, a muted note that says how to read it, a toolbar
//! (scale, focus, uncertainty, zoom, export, expand) and the chart. The card shares its state
//! with the chart inside through [`CardCtx`].

use explorer_core::scale::ScaleKind;
use leptos::prelude::*;
use leptos::svg;

use super::export;
use crate::state::{ScalePref, Uncertainty};
use crate::widgets::{Icon, IconKind, Popover};

/// The state a card shares with its chart.
#[derive(Clone, Copy)]
pub struct CardCtx {
    /// The card overrides the value scale.
    pub scale: RwSignal<Option<ScaleKind>>,
    /// The card overrides outlier focus.
    pub focus: RwSignal<Option<bool>>,
    /// The card overrides the uncertainty.
    pub uncertainty: RwSignal<Option<Uncertainty>>,
    /// The chart has a value axis whose scale the toolbar may switch.
    pub has_scale: RwSignal<bool>,
    /// The default scale of that axis.
    pub scale_default: RwSignal<ScaleKind>,
    /// The chart shows intervals whose kind the toolbar may switch.
    pub has_spread: RwSignal<bool>,
    /// The chart is zoomed.
    pub zoomed: RwSignal<bool>,
    /// Counts zoom commands of the toolbar: reset, in, out.
    pub reset: RwSignal<u32>,
    pub zoom_in: RwSignal<u32>,
    pub zoom_out: RwSignal<u32>,
    /// The framework under the pointer, which the chart highlights.
    pub highlight: RwSignal<Option<String>>,
    /// The SVG of the chart, for export.
    pub svg: NodeRef<svg::Svg>,
    /// The shown data as CSV, for export.
    pub csv: RwSignal<String>,
    pub title: StoredValue<String>,
    /// How to read the chart; the chart may set it.
    pub note: RwSignal<String>,
    /// The explorer of the page, taken when the card is made.
    pub explorer: Option<crate::state::Explorer>,
    /// The likely width of the chart, for the server render.
    pub width_hint: f64,
}

impl CardCtx {
    /// Returns the value scale: the card's choice, else the global one, else the default.
    pub fn scale_kind(&self, default: ScaleKind) -> ScaleKind {
        let global = self
            .explorer
            .map_or(ScalePref::Auto, |e| e.prefs.scale.get());
        self.scale.get().unwrap_or(match global {
            ScalePref::Auto => default,
            ScalePref::Linear => ScaleKind::Linear,
            ScalePref::Log => ScaleKind::Log,
        })
    }

    /// Returns whether outlier focus is on.
    pub fn focus_on(&self) -> bool {
        self.focus
            .get()
            .unwrap_or_else(|| self.explorer.is_some_and(|e| e.prefs.focus.get()))
    }

    /// Returns the uncertainty to show.
    pub fn uncertainty(&self) -> Uncertainty {
        self.uncertainty.get().unwrap_or_else(|| {
            self.explorer
                .map_or(Uncertainty::Auto, |e| e.prefs.uncertainty.get())
        })
    }
}

/// Returns the context of the enclosing card, or a fresh one outside a card.
pub fn use_card() -> CardCtx {
    use_context::<CardCtx>().unwrap_or_else(|| new_ctx(String::new(), false))
}

/// The likely chart width of a half-width card and of a full-width card in the content column.
const HALF: f64 = 664.0;
const FULL: f64 = 1372.0;

fn new_ctx(title: String, wide: bool) -> CardCtx {
    let explorer = use_context::<crate::state::Explorer>();
    CardCtx {
        scale: RwSignal::new(None),
        focus: RwSignal::new(None),
        uncertainty: RwSignal::new(None),
        has_scale: RwSignal::new(false),
        scale_default: RwSignal::new(ScaleKind::Linear),
        has_spread: RwSignal::new(false),
        zoomed: RwSignal::new(false),
        reset: RwSignal::new(0),
        zoom_in: RwSignal::new(0),
        zoom_out: RwSignal::new(0),
        highlight: use_context::<crate::state::Explorer>()
            .map_or_else(|| RwSignal::new(None), |e| e.highlight),
        svg: NodeRef::new(),
        csv: RwSignal::new(String::new()),
        title: StoredValue::new(title),
        note: RwSignal::new(String::new()),
        explorer,
        width_hint: if wide { FULL } else { HALF },
    }
}

/// A chart card.
#[component]
pub fn ChartCard(
    #[prop(into)] title: String,
    /// How to read the chart, such as what whiskers show.
    #[prop(optional, into)]
    note: Signal<String>,
    /// Extra controls in the toolbar.
    #[prop(optional)]
    controls: Option<ViewFn>,
    /// The card spans the whole grid row.
    #[prop(optional)]
    wide: bool,
    children: Children,
) -> impl IntoView {
    let ctx = new_ctx(title.clone(), wide);
    provide_context(ctx);
    Effect::new(move |_| {
        let n = note.get();
        if !n.is_empty() {
            ctx.note.set(n);
        }
    });
    let expanded = RwSignal::new(false);
    let file = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>();
    let file = StoredValue::new(file);
    view! {
        <section class="gx-card gx-chart" data-wide=wide.then_some("") data-expanded=move || expanded.get().then_some("")>
            <header class="gx-chart__head">
                <h3 class="gx-chart__title" title=title.clone()>{title.clone()}</h3>
                <div class="gx-chart__tools">
                    {controls.map(|c| c.run())}
                    {move || ctx.has_spread.try_get().unwrap_or(false).then(|| view! {
                        <select
                            class="gx-chart__select"
                            title="What whiskers and bands show in this chart"
                            on:change=move |ev| ctx.uncertainty.set(Some(Uncertainty::parse(&event_target_value(&ev))))
                            prop:value=move || ctx.uncertainty().key().to_owned()
                        >
                            {Uncertainty::ALL.iter().map(|u| view! { <option value=u.key()>{u.label()}</option> }).collect_view()}
                        </select>
                    })}
                    {move || ctx.has_scale.try_get().unwrap_or(false).then(|| view! {
                        <ScaleSwitch ctx=ctx/>
                        <button
                            type="button"
                            class="gx-iconbtn"
                            title="Focus: fit the axis to the bulk, pin outliers to the edge"
                            data-active=move || ctx.focus_on().then_some("")
                            on:click=move |_| ctx.focus.set(Some(!ctx.focus_on()))
                        >
                            <Icon kind=IconKind::Focus/>
                        </button>
                    })}
                    <button type="button" class="gx-iconbtn" title="Zoom in (or ctrl+wheel)" on:click=move |_| ctx.zoom_in.update(|n| *n += 1)>
                        <Icon kind=IconKind::ZoomIn/>
                    </button>
                    <button type="button" class="gx-iconbtn" title="Zoom out" on:click=move |_| ctx.zoom_out.update(|n| *n += 1)>
                        <Icon kind=IconKind::ZoomOut/>
                    </button>
                    <button
                        type="button"
                        class="gx-iconbtn"
                        title="Reset zoom (or double-click)"
                        disabled=move || !ctx.zoomed.get()
                        on:click=move |_| ctx.reset.update(|n| *n += 1)
                    >
                        <Icon kind=IconKind::Reset/>
                    </button>
                    <Popover right=true class="gx-iconbtn" title="Export" button=ViewFn::from(|| view! { <Icon kind=IconKind::Download/> })>
                        <div class="gx-menu">
                            <button type="button" class="gx-menu__item" on:click=move |_| export::save_svg(ctx, &file.get_value(), false)>"SVG"</button>
                            <button type="button" class="gx-menu__item" on:click=move |_| export::save_svg(ctx, &file.get_value(), true)>"SVG with embedded font"</button>
                            <button type="button" class="gx-menu__item" on:click=move |_| export::save_png(ctx, &file.get_value())>"PNG (2×)"</button>
                            <button type="button" class="gx-menu__item" on:click=move |_| export::save_csv(ctx, &file.get_value())>"CSV of the shown data"</button>
                            <button type="button" class="gx-menu__item" on:click=move |_| export::copy_csv(ctx)>"Copy CSV"</button>
                        </div>
                    </Popover>
                    <button
                        type="button"
                        class="gx-iconbtn"
                        title="Expand"
                        data-active=move || expanded.get().then_some("")
                        on:click=move |_| expanded.update(|e| *e = !*e)
                    >
                        <Icon kind=IconKind::Expand/>
                    </button>
                </div>
            </header>
            <p class="gx-chart__note">{move || ctx.note.get()}</p>
            <div class="gx-chart__body">{children()}</div>
        </section>
        {move || expanded.try_get().unwrap_or(false).then(|| view! {
            <div class="gx-chart__scrim" on:click=move |_| expanded.set(false)></div>
        })}
    }
}

#[component]
fn ScaleSwitch(ctx: CardCtx) -> impl IntoView {
    let is_log = Memo::new(move |_| ctx.scale_kind(ctx.scale_default.get()) == ScaleKind::Log);
    view! {
        <div class="gx-seg gx-seg--tiny" title="Value scale of this chart">
            <button type="button" class="gx-seg__opt" data-active=move || (!is_log.get()).then_some("") on:click=move |_| ctx.scale.set(Some(ScaleKind::Linear))>"lin"</button>
            <button type="button" class="gx-seg__opt" data-active=move || is_log.get().then_some("") on:click=move |_| ctx.scale.set(Some(ScaleKind::Log))>"log"</button>
        </div>
    }
}
