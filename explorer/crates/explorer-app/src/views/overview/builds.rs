//! The builds of a run: every framework and edition with the size of its binary and the parts
//! its app builds by hand, since the framework has none.

use std::sync::Arc;

use explorer_core::api::RunOverview;
use explorer_core::units::format_value;
use leptos::prelude::*;

use crate::palette;
use crate::widgets::Badge;

/// The builds card of a run.
#[component]
pub fn Builds(overview: Arc<RunOverview>) -> impl IntoView {
    let mut builds = overview.provenance.builds.clone();
    builds.sort_by(|a, b| {
        palette::rank(&a.framework)
            .total_cmp(&palette::rank(&b.framework))
            .then_with(|| a.framework.cmp(&b.framework))
    });
    view! {
        <section class="gx-card gx-builds">
            <h3 class="gx-chart__title">"Builds and hand-built parts"</h3>
            <p class="gx-chart__note">
                "An edition is another build of a framework's app · a flagged part is one the app builds by hand; charts draw its measurements hollow (menu: open the menu · spinner: live values · list: the whole list scenario)"
            </p>
            <table class="gx-table">
                <thead><tr><th>"framework"</th><th>"app"</th><th>"binary"</th><th>"built for size"</th><th>"built by hand"</th></tr></thead>
                <tbody>
                    {builds.into_iter().map(|b| {
                        let edition = palette::is_edition(&b.framework);
                        view! {
                            <tr>
                                <td style:color=palette::ink(&b.framework)>
                                    <b>{b.framework.clone()}</b>
                                    {edition.then(|| view! { " " <Badge text=format!("edition of {}", palette::base(&b.framework)) tone="info"/> })}
                                </td>
                                <td class="gx-muted">{format!("apps/{}", b.app)}</td>
                                <td>{b.binary_bytes.map_or("–".to_owned(), |v| format_value(v as f64, "bytes"))}</td>
                                <td>{b.min_binary_bytes.map_or("–".to_owned(), |v| format_value(v as f64, "bytes"))}</td>
                                <td class="gx-builds__flags">
                                    {if b.flags.is_empty() {
                                        view! { <span class="gx-muted">"none"</span> }.into_any()
                                    } else {
                                        b.flags.iter().map(|f| view! {
                                            <div><Badge text=f.part.clone() tone="warn"/>" "<span class="gx-muted">{f.note.clone()}</span></div>
                                        }).collect_view().into_any()
                                    }}
                                </td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </section>
    }
}
