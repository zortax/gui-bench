//! Values by unit: the display unit and factor of a report unit, and number formatting with
//! thousands separators, as the gui-bench plots print them.

mod format;
mod time;

pub use format::{Display, compact, display, format_value, short, thousands};
pub use time::{date_time, duration};
