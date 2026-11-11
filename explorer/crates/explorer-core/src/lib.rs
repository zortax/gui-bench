//! Pure building blocks of the explorer: the wire types that the server functions exchange, the
//! statistics behind uncertainty and comparison, the scale and tick math of the charts, and the
//! formatting of values by unit. Nothing here does IO, so both the server and the browser use it.

#![forbid(unsafe_code)]

pub mod api;
pub mod scale;
pub mod stats;
pub mod units;
