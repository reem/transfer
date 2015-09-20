use std::sync::Arc;

use mio::{EventLoop, EventSet, TryRead};
use mio::tcp::TcpStream;
use eventual::Sender;

use appendbuf::AppendBuf;

use rt::loophandler::{LoopHandler, InnerIoMachine, EventMachine};
use rt::Metadata;

use http::parser::{self, FrameHeader, Frame};
use http;

use prelude::*;
use Handler as HttpHandler;

const FRAME_PAYLOAD_MAX_LENGTH: usize = 1024 * 16;
const FRAME_HEADER_LENGTH: usize = 9;

pub struct ReadEvidence;

pub struct Connection {
    pub connection: TcpStream,
    frames: Future<Sender<Frame, Error>, ()>,
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

        // Parse as many frames as we can.
        loop {
            if let Some(current) = self.io.current {
                let frame = Frame::parse(current,
                                         self.io.buffer.slice().slice_from(9));

                match frame {
                    Err(parser::Error::Incomplete) => break,
                    Err(_) => return None,
                    Ok(frame) => {
                        // Send the frame.
                        self.io.frames = self.io.frames.and_then(|sender| {
                            sender.send(frame)
                        });

                        // Recycle self.io.current and self.io.buffer.
                        self.io.current = None;

                        let mut newbuffer = buf();
                        newbuffer.fill(&self.io.buffer[current.length as usize..]);
                        self.io.buffer = newbuffer;
                    }
                }
            } else {
                let header = FrameHeader::parse(&self.io.buffer);

                match header {
                    Err(::http2parse::Error::Short) => break,
                    Err(_) => return None,
                    Ok(header) => {
                        self.io.current = Some(header);
                    }
                }
            }
        }

        Some(self)
    }

    fn writable(self, event_loop: &mut EventLoop<LoopHandler>,
                handler: &mut LoopHandler) -> Option<Self> {
        Some(self)
    }
}

fn buf() -> AppendBuf {
    AppendBuf::new(FRAME_HEADER_LENGTH + FRAME_PAYLOAD_MAX_LENGTH)
}

impl Connection {
    pub fn new(connection: TcpStream,
               handler: Arc<Box<HttpHandler>>,
               metadata: Metadata) -> Connection {
        let (frames_tx, frames_rx) = Stream::pair();

        http::http(frames_rx, metadata.clone());

        Connection {
            connection: connection,
            frames: Future::of(frames_tx),
            current: None,
            buffer: buf()
        }
    }
}

