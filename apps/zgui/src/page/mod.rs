//! The page of the header scenarios: a header with the status text and a slot on the right, and
//! a scroll container of rows below it. Rows here have one background and no hover color.

mod header;
mod row;

pub use header::HeaderProps;
pub use row::PlainRowProps;

/// The look of the header, the rows, the spinner and the search field.
pub const SHEET: &str = include_str!("page.css");
