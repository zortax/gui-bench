//! The rows scenario: a full-window vertical scroll container that holds every row as a real
//! view, and scrolls 1:1 with the input in logical pixels.

mod list;
pub mod row;
pub mod wheel;

pub use list::view;
