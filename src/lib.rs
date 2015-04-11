#![feature(core, std_misc, box_syntax)]
#![cfg_attr(test, deny(warnings))]
#![cfg_attr(test, feature(test))]
#![allow(unused_variables)]
// #![deny(missing_docs)]

//! # Transfer

extern crate mio;
extern crate hyper;
extern crate httparse;
extern crate iobuf;
extern crate syncbox;
extern crate eventual;

pub use eventual::{Future, Complete, Stream, Sender};

#[macro_use]
extern crate debug_unreachable;

pub use error::{Result, Error};

pub trait Handler: Send + Sync + 'static {}

pub mod prelude {
    pub use eventual::{Future, Stream, Join, Async, Select};
    pub use {Result, Error, Handler};
}

pub mod http;

/// Falcon's Error type and associated impls.
pub mod error;

/// Falcon's runtime, including the event loop.
pub mod rt;

