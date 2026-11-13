//! The steps that carry the story: the responses and the pacing measurements of the app
//! scenario, the part of the app each one exercises, and the budgets that judge them.

/// A step of a scenario: its report and what the user does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    /// The report, such as `hover-row`.
    pub name: &'static str,
    /// What the user does.
    pub title: &'static str,
}

const fn step(name: &'static str, title: &'static str) -> Step {
    Step { name, title }
}

/// The responses of the app scenario.
pub const RESPONSES: [Step; 10] = [
    step("hover-row", "Hover a table row"),
    step("hover-nav", "Hover a nav item"),
    step("key-type", "Type a character"),
    step("click-compare", "Toggle the chart compare"),
    step("click-menu", "Open the menu"),
    step("click-sort", "Sort the table"),
    step("click-filter", "Filter by status"),
    step("key-search", "Search: hide every row"),
    step("page-messages", "Switch to the messages"),
    step("page-overview", "Switch to the overview"),
];

/// The pacing measurements of the app scenario.
pub const PACED: [Step; 3] = [
    step("scroll", "Scroll, touchpad glide"),
    step("scroll-fling", "Scroll, fling"),
    step("live", "Live values and spinner"),
];

/// The frame budgets: a refresh rate in Hz and one refresh in milliseconds.
pub const BUDGETS: [(u32, f64); 2] = [(144, 1000.0 / 144.0), (60, 1000.0 / 60.0)];

/// The share of missed refreshes up to which a scroll or an animation counts as smooth.
pub const SMOOTH_SHARE: f64 = 0.01;

/// The median hover response in milliseconds above which the larger sizes of a framework do
/// not run.
pub const STOP_MS: f64 = 250.0;

/// The part that every measurement of the list scenario exercises.
pub const LIST_PART: &str = "list";

/// Returns the part of the app that a measurement of a scenario exercises, where an app may
/// build that part by hand.
pub fn part_of(scenario: &str, measurement: &str) -> Option<&'static str> {
    match (scenario, measurement) {
        ("list", _) => Some(LIST_PART),
        (_, "click-menu") => Some("menu"),
        (_, "live") => Some("spinner"),
        _ => None,
    }
}

/// Returns what the user does in a step, or the name of the step.
pub fn title_of(measurement: &str) -> &str {
    RESPONSES
        .iter()
        .chain(&PACED)
        .find(|s| s.name == measurement)
        .map_or(measurement, |s| s.title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts() {
        assert_eq!(part_of("app", "click-menu"), Some("menu"));
        assert_eq!(part_of("app", "live"), Some("spinner"));
        assert_eq!(part_of("app", "hover-row"), None);
        assert_eq!(part_of("list", "hover-row"), Some("list"));
    }
}
