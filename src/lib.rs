#![cfg_attr(test, deny(warnings))]
#![cfg_attr(test, feature(test))]
// #![deny(missing_docs)]

//! # Falcon

extern crate mio;
extern crate hyper;
extern crate httparse;
extern crate iobuf;
extern crate syncbox;

pub use error::{Result, Error};

/// Falcon's Error type and associated impls.
pub mod error;

/// Falcon's runtime, including the event loop.
pub mod rt;

