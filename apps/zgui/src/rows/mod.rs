//! The rows scenario: one scroll container that fills the window and holds N real rows.

mod list;
mod row;

pub use list::RowsProps;
pub use row::RowProps;

/// The look of the window, the scroll container and every row.
pub const SHEET: &str = include_str!("rows.css");
