use rt;

pub use self::stream::Stream;
pub use self::error::{Error, Result};

pub mod parser;
pub mod stream;
pub mod error;

use self::parser::{Frame, StreamIdentifier};

use std::collections::HashMap;

use eventual::Async;

pub fn http(frames: ::eventual::Stream<parser::Frame, ::Error>,
            metadata: rt::Metadata) {
    let mut http2 = Http2::new();

    frames.map_async(move |frame| {
        Ok(try!(http2.apply(frame)))
    }).fire();
}

#[derive(Debug, Default)]
pub struct Http2 {
    streams: HashMap<StreamIdentifier, Option<Stream>>
}

impl Http2 {
    pub fn new() -> Http2 { Http2::default() }

    pub fn apply(&mut self, frame: Frame) -> Result<()> {
        let id = frame.header.id;

        let stream = self.streams.entry(id)
            .or_insert_with(|| Some(Stream::new(id)))
            .take().expect("Recursively applied frame to stream.");

        let stream = try!(stream.apply(self, frame));

        self.streams.insert(id, Some(stream));

        Ok(())
    }
}

