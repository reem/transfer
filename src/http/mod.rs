pub use self::stream::Stream;
pub use self::error::{Error, Result};

pub mod parser;
pub mod stream;
pub mod error;
pub mod encoder;

use self::parser::{Frame, StreamIdentifier};
use self::encoder::FrameEncoder;

use std::collections::{VecDeque, HashMap};
use std::boxed::FnBox;
use std::fmt;

use eventual::Async;

#[derive(Debug, Default)]
pub struct Http2 {
    streams: HashMap<StreamIdentifier, Option<Stream>>,
    // TODO(reem): Replace with a priority/dependency tree.
    outgoing: VecDeque<(Frame, WriteCallback)>
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

    pub fn get_next_encoder(&mut self) -> Option<(FrameEncoder, WriteCallback)> {
        self.outgoing.pop_back().map(|(frame, cb)| (FrameEncoder::from(frame), cb))
    }

    pub fn queue_outgoing_frame<F>(&mut self, frame: Frame, cb: F)
    where F: for<'a> FnBox<&'a mut Http2, Output=()> + Send + 'static {
        self.outgoing.push_front((frame, WriteCallback(Box::new(cb))))
    }
}

pub struct WriteCallback(pub Box<for<'a> FnBox<&'a mut Http2, Output=()> + Send>);

impl fmt::Debug for WriteCallback {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Write Callback")
    }
}

