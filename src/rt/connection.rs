use std::sync::Arc;

use mio::{NonBlock, EventLoop, ReadHint, Token};
use mio::tcp::TcpStream;
use iobuf::{Allocator, AROIobuf, AppendBuf};
use eventual::Sender;

use rt::loophandler::LoopHandler;

use prelude::*;
use Handler as HttpHandler;

struct Response;
struct BodyMessage;

pub struct Connection {
    stream: NonBlock<TcpStream>,

    // Communication with the handling actors.
    readbuffer: AppendBuf<'static>,
    snapshots: Sender<Snapshot, Error>,
    responses: Stream<Response, Error>,

    // Metadata
    handler: Arc<Box<HttpHandler>>,
    allocator: Arc<Box<Allocator>>,
    executor: Arc<Box<Run + Send + Sync>>
}

pub enum Snapshot {
    Head(AROIobuf),
    Body(BodyMessage)
}

impl Connection {
    pub fn new(stream: NonBlock<TcpStream>,
               handler: Arc<Box<HttpHandler>>,
               allocator: Arc<Box<Allocator>>,
               executor: Arc<Box<Run + Send + Sync>>) -> Connection {
        let readbuffer = AppendBuf::new_with_allocator(16 * 1024, allocator.clone());

        let (snapshots_tx, spanshots_rx) = Stream::pair();
        let (responses_tx, responses_rx) = Stream::pair();

        Connection {
            stream: stream,

            readbuffer: readbuffer,
            snapshots: snapshots_tx,
            responses: responses_rx,

            handler: handler,
            allocator: allocator,
            executor: executor
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

