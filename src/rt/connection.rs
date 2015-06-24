use std::sync::Arc;

use mio::{EventLoop, ReadHint, Token};
use mio::tcp::TcpStream;
use iobuf::{AROIobuf, AppendBuf};
use eventual::{Sender, BusySender};

use rt::loophandler::{LoopHandler, Registration};
use rt::{Executor, Metadata};

use prelude::*;
use Handler as HttpHandler;

pub const MAX_REQUEST_HEAD_LENGTH: usize = 16 * 1024;

pub struct ReadEvidence;

pub struct Connection {
    stream: TcpStream,
    metadata: Metadata,
}

impl Connection {
    pub fn new(stream: TcpStream,
               handler: Arc<Box<HttpHandler>>,
               metadata: Metadata) -> Connection {
        let readbuffer = AppendBuf::new_with_allocator(MAX_REQUEST_HEAD_LENGTH,
                                                       metadata.allocator.clone());

    }

    pub fn readable(handler: &mut LoopHandler,
                    event_loop: &mut EventLoop<LoopHandler>,
                    token: Token, hint: ReadHint) {
        if let &mut Registration::Connection(ref mut connection) = &mut handler.slab[token] {
        } else {
            unreachable!("LoopHandler yielded acceptor to connection.");
        }
    }

    pub fn writable(handler: &mut LoopHandler,
                    event_loop: &mut EventLoop<LoopHandler>,
                    token: Token) {

    }
}
