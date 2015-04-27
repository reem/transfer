use std::sync::Arc;

use mio::{NonBlock, EventLoop, ReadHint, Token};
use mio::tcp::TcpStream;
use iobuf::{AROIobuf, AppendBuf};
use eventual::Sender;

use rt::loophandler::LoopHandler;
use rt::{Executor, Metadata};

use http::parser::{RawRequest, RawResponse};
use http::{Request, Response};

use prelude::*;
use Handler as HttpHandler;

pub const MAX_REQUEST_HEAD_LENGTH: usize = 16 * 1024;

pub struct ReadEvidence;

pub struct Connection {
    stream: NonBlock<TcpStream>,
    metadata: Metadata,
    requests: Sender<(RawRequest, RawResponse), Error>,
    state: ConnectionState
}

enum ConnectionState {
    Head(AppendBuf<'static>),
    ReadyBody(Sender<ReadEvidence, Error>),
    BusyBody(BusySender<ReadEvidence, Error>)
}

impl Connection {
    pub fn new(stream: NonBlock<TcpStream>,
               handler: Arc<Box<HttpHandler>>,
               metadata: Metadata) -> Connection {
        let readbuffer = AppendBuf::new_with_allocator(MAX_REQUEST_HEAD_LENGTH,
                                                       metadata.allocator.clone());

        let (requests_tx, requests_rx) = Stream::pair();

        // Set up the connection to be handled properly when the stream is completed.
        // Each request will be fired on the rt executor, and *only one* request will be
        // handled at once.
        //
        // If more than one request is yielded from the same connection at the same time
        // semantic errors can easily occur due to the stream-of-evidence style used
        // for request bodies.
        let executor = metadata.executor.clone();
        requests_rx.map_async(move |(raw_req, raw_res)| {
            executor.invoke(move || {
                handler.handle(Request::from_raw(raw_req), Response::from_raw(raw_res));
            })
        }).fire();

        Connection {
            stream: stream,
            metadata: metadata,
            requests: requests_tx,
            state: ConnectionState::Head(readbuffer)
        }
    }

    pub fn readable(handler: &mut LoopHandler,
                    event_loop: &mut EventLoop<LoopHandler>,
                    token: Token, hint: ReadHint) {

    }

    pub fn writable(handler: &mut LoopHandler,
                    event_loop: &mut EventLoop<LoopHandler>,
                    token: Token) {

    }
}

