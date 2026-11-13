//! The top bar: the title of the page, the search input, the live switch with its spinner, the
//! export menu and the avatar.

use zgui::prelude::*;
use zgui::reactive::UnsyncCallback;
use zgui_ui::prelude::*;

use super::live::Live;
use super::{Filters, spec};

/// The white bar above the content. Typing in the search writes `filters.search`; the switch
/// starts and stops the live updates.
#[component]
pub fn TopBar(filters: Filters, active: RwSignal<usize>, live: Live) -> impl IntoView {
    let timers = Timers::current();
    let on_live = UnsyncCallback::new(move |on: bool| live.run(timers.as_ref(), on));
    view! {
        row(class = "topbar") {
            text(class = "topbar__title") {{move || spec::NAV_ITEMS[active.get()]}}
            Input(
                value = filters.search,
                placeholder = spec::SEARCH_PLACEHOLDER,
                label = spec::SEARCH_PLACEHOLDER,
                class = "search"
            )
            Switch(checked = live.on, on_change = on_live, class = "switch live", a11y:label = spec::LIVE_LABEL)
            box(class = "spinner", class:spinning = move || live.on.get())
            ExportMenu()
            row(class = "topbar__avatar") {{spec::USER_INITIALS}}
        }
    }
}

/// The export button and the menu it opens.
#[component]
fn ExportMenu() -> impl IntoView {
    view! {
        DropdownMenu {
            DropdownMenuTrigger(class = "export") {{spec::EXPORT_LABEL}}
            DropdownMenuContent(class = "export__menu") {
                {Vec::from(spec::MENU_ITEMS.map(|label| view! {
                    MenuItem(class = "export__item") {{label}}
                }.into_view()))}
            }
        }
    }
}
