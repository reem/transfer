#![cfg_attr(test, deny(warnings))]
#![cfg_attr(test, feature(test))]
#![deny(missing_docs)]

//! # Falcon

extern crate mio;
extern crate hyper;
extern crate httparse;
extern crate iobuf;
extern crate syncbox;

/// Falcon's Error type and associated impls.
pub mod error;

