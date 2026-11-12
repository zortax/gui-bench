//! The rows scenario: a full-window vertical scroll container that holds every row as a real
//! view, and scrolls as floem's scroll container does by default.

mod list;
pub mod row;

pub use list::view;
