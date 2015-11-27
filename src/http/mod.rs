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
    pub outgoing: Outgoing
}

#[derive(Debug, Default)]
pub struct Outgoing {
    // TODO(reem): Replace with a priority/dependency tree.
    queue: VecDeque<(Frame, WriteCallback)>
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

impl Outgoing {
    /// Queue a frame for writing.
    ///
    /// The callback will be called when the frame has been fully written
    /// to the socket.
    pub fn enqueue<F>(&mut self, frame: Frame, cb: F)
    where F: for<'a> FnBox<(&'a mut Http2,), Output=()> + Send + 'static {
        self.queue.push_front((frame, WriteCallback(Box::new(cb))))
    }

    /// Dequeue an encoder for writing to a socket.
    ///
    /// Note: ensure that the callback is called when the encoder is finished.
    pub fn dequeue(&mut self) -> Option<(FrameEncoder, WriteCallback)> {
        self.queue.pop_back().map(|(frame, cb)| (FrameEncoder::from(frame), cb))
    }

    /// Are there any frames remaining to be encoded?
    pub fn is_empty(&self) -> bool { self.queue.is_empty() }
}

pub struct WriteCallback(pub Box<for<'a> FnBox<(&'a mut Http2,), Output=()> + Send>);

impl fmt::Debug for WriteCallback {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Write Callback")
    }
}

