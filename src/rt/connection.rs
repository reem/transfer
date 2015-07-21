use std::sync::Arc;
use std::collections::HashMap;
use std::io::{self, Read, Write};

use mio::{EventLoop, Token};
use mio::tcp::TcpStream;
use iobuf::{AROIobuf, AppendBuf};
use eventual::{Sender, BusySender};

use rt::loophandler::{LoopHandler, Registration};
use rt::{Executor, Metadata};

use http::parser::{FrameHeader, StreamIdentifier};

use prelude::*;
use Handler as HttpHandler;

pub struct ReadEvidence;

pub struct Connection {
    connection: TcpStream,
    streams: HashMap<StreamIdentifier, ::http::Stream>,
    metadata: Metadata,
    waiting: Option<FrameHeader>,
    buffer: AppendBuf<'static>
}

impl Connection {
    pub fn new(connection: TcpStream,
               handler: Arc<Box<HttpHandler>>,
               metadata: Metadata) -> Connection {
        let readbuffer = AppendBuf::new_with_allocator(9, metadata.allocator.clone());

        Connection {
            connection: connection,
            streams: HashMap::new(),
            metadata: metadata,
            waiting: None,
            buffer: readbuffer
        }
    }

    pub fn readable(handler: &mut LoopHandler,
                    event_loop: &mut EventLoop<LoopHandler>,
                    token: Token, last_event: bool) {
        use std::io::ErrorKind;

        if let &mut Registration::Connection(ref mut connection) = &mut handler.slab[token] {
            loop {
                let remove = match connection.waiting {
                    Some(header) => {
                        match connection.connection.read(unsafe { connection.buffer.as_mut_window_slice() }) {
                            Ok(0) => last_event,
                            Ok(n) => {
                                false
                            },
                            Err(e) => {
                                if e.kind() == ErrorKind::WouldBlock {
                                    false
                                } else {
                                    true
                                }
                            },
                        }
                    },
                    None => {
                        false
                    }
                };
            }
        } else {
            unreachable!("LoopHandler yielded acceptor to connection.");
        }
    }

    pub fn writable(handler: &mut LoopHandler,
                    event_loop: &mut EventLoop<LoopHandler>,
                    token: Token, last_event: bool) {

    }
}
