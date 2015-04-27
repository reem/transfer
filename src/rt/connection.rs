use std::sync::Arc;

use mio::{NonBlock, EventLoop, ReadHint, Token};
use mio::tcp::TcpStream;
use iobuf::{AROIobuf, AppendBuf};
use eventual::{Sender, BusySender};

use rt::loophandler::{LoopHandler, Registration};
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
    state: Option<ConnectionState> // Always Some
}

enum ConnectionState {
    Head(AppendBuf<'static>),
    ReadyBody(Sender<ReadEvidence, Error>),
    BusyBody(BusySender<ReadEvidence, Error>),
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
            let handler = handler.clone();
            executor.invoke(Box::new(move || {
                // TODO:
                // In order to maintain the invariant that a request must be entirely
                // read out and a full response written before the next request is
                // queued, the Request and Response types will have to register
                // some form of "completed" Futures, which will be joined here rather
                // then the current setup.
                Ok(handler.handle(Request::from(raw_req), Response::from(raw_res)))
            }))
        }).fire();

        Connection {
            stream: stream,
            metadata: metadata,
            requests: requests_tx,
            state: Some(ConnectionState::Head(readbuffer))
        }
    }

    pub fn readable(handler: &mut LoopHandler,
                    event_loop: &mut EventLoop<LoopHandler>,
                    token: Token, hint: ReadHint) {
        if let &mut Registration::Connection(ref mut connection) = &mut handler.slab[token] {
            connection.state = Some(match connection.state.take().unwrap() {
                ConnectionState::Head(readbuffer) => {
                    // TODO:
                    // Read into the readbuffer as much as possible, since we may be
                    // running under edge polling.

                    panic!("Unimplemented: read the request head.")
                },

                ConnectionState::ReadyBody(evidence_stream) => {
                    // TODO:
                    // Implement HTTP body rules, to know when to switch back to
                    // ConnectionState::Head for the next request.
                    ConnectionState::BusyBody(evidence_stream.send(ReadEvidence))
                },

                ConnectionState::BusyBody(busy) => {
                    if busy.is_ready() {
                        // TODO: Handle the unwrap on the following line.
                        ConnectionState::BusyBody(busy.await().unwrap().send(ReadEvidence))
                    } else {
                        // Can only occur under level polling, since under edge polling
                        // the read evidence must be used completely before this connection
                        // will be notified it is readable again.
                        //
                        // Under level conditions, we can just ignore the hint until the
                        // previous hint is processed.
                        ConnectionState::BusyBody(busy)
                    }
                }
            })
        } else {
            unsafe { debug_unreachable!("LoopHandler yielded acceptor to connection.") }
        }
    }

    pub fn writable(handler: &mut LoopHandler,
                    event_loop: &mut EventLoop<LoopHandler>,
                    token: Token) {

    }
}

