//! The response view: the response time of every interaction of the app against its size, and
//! what each 1,000 elements more cost.

mod charts;
mod slope;
mod view;

pub use view::ResponseView;
