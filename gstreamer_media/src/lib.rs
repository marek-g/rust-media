extern crate failure;
extern crate gstreamer;
extern crate gstreamer_app;
extern crate glib;

pub type Result<T> = std::result::Result<T, failure::Error>;

mod pipeline_factory;
pub use pipeline_factory::*;
