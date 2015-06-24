#![feature(box_syntax, raw, thunk)]
#![cfg_attr(test, deny(warnings))]
#![cfg_attr(test, feature(test))]
#![allow(unused_variables)]
// #![deny(missing_docs)]

//! # Transfer

extern crate mio;
extern crate http2parse;
extern crate iobuf;
extern crate syncbox;
extern crate eventual;

pub use eventual::{Future, Complete, Stream, Sender};

#[macro_use(debug_unreachable)]
extern crate debug_unreachable;

pub use error::{Result, Error};

pub trait Handler: Send + Sync + 'static {
    fn handle(&self);
}

pub mod prelude {
    pub use eventual::{Future, Stream, Join, Async, Select};
    pub use iobuf::Iobuf;
    pub use {Result, Error, Handler};
}

pub mod http;

/// Transfer's Error type and associated impls.
pub mod error;

/// Transfer's runtime, including the event loop.
pub mod rt;

mod util;

