pub use self::stream::Stream;
pub use self::error::{Error, Result};

pub mod parser;
pub mod stream;
pub mod error;
pub mod encoder;

use self::parser::{Frame, StreamIdentifier};

use std::collections::HashMap;

use eventual::Async;

#[derive(Debug, Default)]
pub struct Http2 {
    streams: HashMap<StreamIdentifier, Option<Stream>>
}

impl Http2 {
    pub fn new() -> Http2 { Http2::default() }

    pub fn stream(&mut self, id: StreamIdentifier) -> Stream {
        self.streams.entry(id)
            .or_insert_with(|| Some(Stream::new(id)))
            .take().expect("Recursively applied frame to stream.")
    }

    pub fn apply(&mut self, frame: Frame) -> Result<()> {
        debug!("Applying frame {:?}", frame);

        let id = frame.header.id;
        let stream = try!(self.stream(id).apply(self, frame));
        self.streams.insert(id, Some(stream));

        Ok(())
    }
}

