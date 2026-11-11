//! The page shell, the router and the frame around every view: the top bar with the run
//! pickers and the theme switch, the sidebar with the filters, and the cell drawer.

mod layout;
mod runbar;
mod shell;
mod sidebar;
mod theme;

pub use shell::{App, shell};
