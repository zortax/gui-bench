//! The data of the spec (`apps/shared/dashboard.rs`) as the JSON the page reads: the texts, the
//! nav items, the metric cards with their live values, the chart series, the statuses, and the
//! orders and messages of the app scenario. The list scenario gets the tables its rows are made
//! from instead of the rows, since it has up to a million of them.

use serde_json::{Value, json};

use crate::cli::Scenario;
use crate::dashboard::{self as spec, Kpi, Message, Order, Page, Status};

/// The period of `kpi_value` in the tick, from tick 1 on.
const LIVE_PERIOD: u64 = 97;

/// Converts a spec color to CSS.
fn css_color(rgb: u32) -> String {
    format!("#{rgb:06x}")
}

/// Returns everything the page needs for `scenario`.
pub fn payload(scenario: Scenario) -> Value {
    let (name, count) = match scenario {
        Scenario::App { count } => ("app", count),
        Scenario::List { count } => ("list", count),
    };
    let app = matches!(scenario, Scenario::App { .. });
    json!({
        "scenario": name,
        "count": count,
        "texts": {
            "appName": spec::APP_NAME,
            "userName": spec::USER_NAME,
            "userRole": spec::USER_ROLE,
            "userInitials": spec::USER_INITIALS,
            "searchPlaceholder": spec::SEARCH_PLACEHOLDER,
            "exportLabel": spec::EXPORT_LABEL,
            "chartTitle": spec::CHART_TITLE,
            "filtersTitle": spec::FILTERS_TITLE,
            "compareLabel": spec::COMPARE_LABEL,
            "resetLabel": spec::RESET_LABEL,
            "applyLabel": spec::APPLY_LABEL,
            "liveLabel": spec::LIVE_LABEL,
            "emptyLabel": spec::EMPTY_LABEL,
        },
        "navItems": nav_items(),
        "kpis": spec::KPIS.iter().enumerate().map(|(k, kpi)| kpi_json(k, kpi)).collect::<Vec<_>>(),
        "liveIntervalMs": spec::LIVE_INTERVAL_MS,
        "segments": spec::SEGMENTS,
        "series": (0..spec::SEGMENTS.len()).map(series).collect::<Vec<_>>(),
        "gridLines": spec::GRID_LINES.map(|y| y - spec::PLOT_TOP),
        "statuses": Status::ALL.map(status),
        "menuItems": spec::MENU_ITEMS,
        "columns": spec::COLUMNS,
        "sortColumn": spec::SORT_COLUMN,
        "orders": if app { spec::orders(count).iter().map(order).collect() } else { Vec::new() },
        "messages": if app { spec::messages(count).iter().map(message).collect() } else { Vec::new() },
        "names": {
            "first": spec::FIRST_NAMES,
            "last": spec::LAST_NAMES,
            "avatars": spec::AVATAR_COLORS.map(css_color),
            // The filter rows of orders 0 to 3; the statuses repeat from there.
            "statuses": (0..4).map(|i| spec::order(i).status.filter_row()).collect::<Vec<_>>(),
        },
    })
}

/// The nav items with their pages and the unread count of the one with the badge.
fn nav_items() -> Vec<Value> {
    spec::NAV_ITEMS
        .iter()
        .enumerate()
        .map(|(item, label)| {
            let page = match spec::page(item) {
                Page::Overview => "overview",
                Page::Messages => "messages",
                Page::Empty => "empty",
            };
            let badge = (item == spec::BADGE_ITEM).then_some(spec::BADGE_COUNT);
            json!({ "label": label, "page": page, "badge": badge })
        })
        .collect()
}

/// Metric card `k`. `values[0]` is the startup value and `values[1 + (tick - 1) % 97]` the value
/// at a live tick from 1 on.
fn kpi_json(k: usize, kpi: &Kpi) -> Value {
    json!({
        "label": kpi.label,
        "delta": kpi.delta,
        "up": kpi.up,
        "spark": kpi.spark,
        "values": (0..=LIVE_PERIOD).map(|tick| spec::kpi_value(k, tick)).collect::<Vec<_>>(),
    })
}

/// The bar heights of a chart series and of its previous period.
fn series(series: usize) -> Value {
    json!({
        "bars": (0..spec::CHART_BARS).map(|i| spec::bar_height(series, i)).collect::<Vec<_>>(),
        "previous": (0..spec::CHART_BARS)
            .map(|i| spec::previous_height(series, i))
            .collect::<Vec<_>>(),
    })
}

/// A status: its label and the colors of its pill.
fn status(status: Status) -> Value {
    let (background, color) = status.colors();
    json!({
        "label": status.label(),
        "background": css_color(background),
        "color": css_color(color),
    })
}

/// An order with its texts; `status` is its filter row, an index into the statuses.
fn order(order: &Order) -> Value {
    json!({
        "number": order.number,
        "numberText": order.number_text(),
        "customer": order.customer,
        "initials": order.initials,
        "avatar": css_color(order.avatar),
        "status": order.status.filter_row(),
        "progress": order.progress,
        "cents": order.cents,
        "amount": order.amount(),
    })
}

/// A message of the feed.
fn message(message: &Message) -> Value {
    json!({
        "author": message.author,
        "initials": message.initials,
        "avatar": css_color(message.avatar),
        "time": message.time,
        "body": message.body,
    })
}
