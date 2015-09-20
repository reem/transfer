use std::sync::Arc;

use mio::{EventLoop, EventSet, TryRead};
use mio::tcp::TcpStream;
use eventual::{Sender};

use appendbuf::AppendBuf;

use rt::loophandler::{LoopHandler, InnerIoMachine, EventMachine};
use rt::{Executor, Metadata};

use http::parser::{FrameHeader, Frame};
use http;

use prelude::*;
use Handler as HttpHandler;

const FRAME_PAYLOAD_MAX_LENGTH: usize = 1024 * 16;
const FRAME_HEADER_LENGTH: usize = 9;

pub struct ReadEvidence;

pub struct Connection {
    pub connection: TcpStream,
    frames: Sender<Frame, Error>,
    metadata: Metadata,
    current: Option<FrameHeader>,
    buffer: AppendBuf
}

impl EventMachine for InnerIoMachine<Connection> {
    fn ready(self, event_loop: &mut EventLoop<LoopHandler>, handler: &mut LoopHandler,
             events: EventSet) -> Option<Self> {
        let mut optself = Some(self);

        if events.contains(EventSet::readable()) {
            optself = optself.and_then(|this| this.readable(event_loop, handler))
        }

        if events.contains(EventSet::writable()) {
            optself = optself.and_then(|this| this.writable(event_loop, handler))
        }

        optself
    }
}

impl InnerIoMachine<Connection> {
    fn readable(mut self, event_loop: &mut EventLoop<LoopHandler>,
                handler: &mut LoopHandler) -> Option<Self> {
        // Read in as much data as we can.
        loop {
            match self.io.connection.try_read(self.io.buffer.get_write_buf()) {
                Ok(Some(n)) => unsafe { self.io.buffer.advance(n) },
                Ok(None) => break,
                Err(e) => {
                    error!("Connection error {:?}", e);
                    return None
                }
            }
        }

        if let Some(current) = self.io.current {

        }

        Some(self)
    }

    fn writable(self, event_loop: &mut EventLoop<LoopHandler>,
                handler: &mut LoopHandler) -> Option<Self> {
        Some(self)
    }
}

impl Connection {
    pub fn new(connection: TcpStream,
               handler: Arc<Box<HttpHandler>>,
               metadata: Metadata) -> Connection {
        let readbuffer =
            AppendBuf::new(FRAME_HEADER_LENGTH + FRAME_PAYLOAD_MAX_LENGTH);

        let (frames_tx, frames_rx) = Stream::pair();

        http::http(frames_rx, metadata.clone());

        Connection {
            connection: connection,
            frames: frames_tx,
            metadata: metadata,
            current: None,
            buffer: readbuffer
        }
    }
}

