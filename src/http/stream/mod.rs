use http::parser::{StreamIdentifier, Frame, Payload};

use self::state::State;
use super::Http2;
use super::error::{Error, Result};

mod state;

/// An http2 "stream", as described in the spec:
///
/// ```text
/// A "stream" is an independent, bidirectional sequence of frames exchanged between the client and
/// server within an HTTP/2 connection. Streams have several important charateristics:
///   * A single HTTP/2 connection can contain multiple concurrently open streams, with either
///     endpoint interleaving frames from multiple streams.
///   * Streams can be established and used unilaterally or shared by either the client or server.
///   * Streams can be closed by either endpoint.
///   * The order in which frames are sent on a stream is significant. Recipients process frames in
///     the order they are received. In particular, the order of HEADERS and DATA frames is
///     semantically significant.
///   * Streams are identified by an integer. Stream identifiers are assigned to streams by the
///     endpoint initiating the stream.
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stream {
    id: StreamIdentifier,
    state: State
}

impl Stream {
    pub fn new(id: StreamIdentifier) -> Stream {
        Stream {
            id: id,
            state: State::default()
        }
    }

    pub fn apply(self, streams: &mut Http2, frame: Frame) -> Result<Self> {
        let header = frame.header;
        let payload = frame.payload;

        // FIXME: remove
        return Ok(self);

        match (self.state, payload) {
            (State::Idle, Payload::Headers { priority, block }) => {
                Ok(self)
            },

            (State::ReservedLocal, Payload::Reset(error)) => {
                Ok(self)
            },

            (State::ReservedRemote, Payload::Headers { priority, block }) => {
                Ok(self)
            },

            (State::ReservedRemote, Payload::Reset(error)) => {
                Ok(self)
            },

            (State::Open, Payload::Headers { priority, block }) => {
                Ok(self)
            },

            (State::Open, Payload::Data(data)) => {
                Ok(self)
            },

            (State::Open, Payload::Reset(error)) => {
                Ok(self)
            },

            (State::HalfClosedLocal, Payload::Reset(error)) => {
                Ok(self)
            },

            (_, Payload::Priority(priority)) => {
                Ok(self)
            },

            // Illegal state/frame combo.
            (state, frame) => {
                Err(Error::InvalidFrameTypeForStreamState)
            }
        }
    }
}

