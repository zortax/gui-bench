//! The table scenario: a header with the row count above a keyed table of rows. The function
//! keys run the table operations of `spec::Op`, a click selects a row, and with a ticker a timer
//! updates rows and the header. Each variant holds the rows its own way.

mod coarse;
mod fine;
mod keys;

pub use coarse::CoarseTableProps;
pub use fine::FineTableProps;
