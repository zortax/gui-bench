//! The rows scenario: one scroll container that fills the window and holds N real rows.

mod list;
mod row;
pub mod wheel;

pub use list::Rows;
pub use row::Row;

/// The look of the window, the scroll container and every row.
pub const CSS: &str = include_str!("rows.css");
