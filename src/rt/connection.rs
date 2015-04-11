use std::sync::Arc;

use mio::{NonBlock, EventLoop, ReadHint, Token};
use mio::tcp::TcpStream;
use iobuf::{AROIobuf, AppendBuf};
use eventual::Sender;

use rt::loophandler::LoopHandler;
use rt::{Executor, Metadata};

use http;
use prelude::*;
use Handler as HttpHandler;

pub struct Response;
pub struct BodyMessage;

pub struct Connection {
    stream: NonBlock<TcpStream>,
    metadata: Metadata,
    readbuffer: AppendBuf<'static>,
    snapshots: Sender<Snapshot, Error>,
    responses: Stream<Response, Error>
}

pub enum Snapshot {
    Head(AROIobuf),
    Body(BodyMessage)
}

impl Connection {
    pub fn new(stream: NonBlock<TcpStream>,
               handler: Arc<Box<HttpHandler>>,
               metadata: Metadata) -> Connection {
        let readbuffer = AppendBuf::new_with_allocator(32 * 1024,
                                                       metadata.allocator.clone());
        let (snapshots_tx, spanshots_rx) = Stream::pair();
        let (responses_tx, responses_rx) = Stream::pair();

        let metadata1 = metadata.clone();
        metadata.executor.execute(Box::new(move || {
            http::handle_connection(metadata1, handler, spanshots_rx, responses_tx);
        }));

        Connection {
            stream: stream,
            metadata: metadata,
            readbuffer: readbuffer,
            snapshots: snapshots_tx,
            responses: responses_rx,
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

