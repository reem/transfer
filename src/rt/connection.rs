use std::sync::Arc;

use mio::{EventLoop, EventSet, PollOpt, TryRead};
use mio::tcp::TcpStream;

use appendbuf::AppendBuf;

use rt::loophandler::{LoopHandler, InnerIoMachine, EventMachine};
use rt::Metadata;

use http::parser::{self, FrameHeader, Frame};
use http::encoder::{FrameEncoder, Encoder, EncodeResult};
use http;

use prelude::*;
use Handler as HttpHandler;

const FRAME_PAYLOAD_MAX_LENGTH: usize = 1024 * 16;
const FRAME_HEADER_LENGTH: usize = 9;

#[derive(Debug)]
pub struct Connection {
    pub connection: TcpStream,
    http2: http::Http2,
    current: Option<FrameHeader>,
    buffer: AppendBuf,
    outgoing: Option<(FrameEncoder, http::WriteCallback)>
}

impl Connection {
    pub fn new(connection: TcpStream,
               handler: Arc<Box<HttpHandler>>,
               metadata: Metadata) -> Connection {
        Connection {
            connection: connection,
            http2: http::Http2::new(),
            current: None,
            buffer: buf(),
            outgoing: None
        }
    }
}

impl EventMachine for InnerIoMachine<Connection> {
    fn ready(self, event_loop: &mut EventLoop<LoopHandler>, handler: &mut LoopHandler,
             events: EventSet) -> Option<Self> {
        let mut optself = Some(self);

        if events.contains(EventSet::readable()) {
            debug!("Readable event received on connection.");
            optself = optself.and_then(|this| this.readable(event_loop, handler))
        }

        if events.contains(EventSet::writable()) {
            debug!("Writable event received on connection.");
            optself = optself.and_then(|this| this.writable(event_loop, handler))
        }

        if let Some(ref this) = optself {
            if this.io.outgoing.is_some() || !this.io.http2.outgoing.is_empty() {
                debug!("Reregistering connection.");
                event_loop.reregister(&this.io, this.token,
                                      EventSet::readable() | EventSet::writable(),
                                      PollOpt::edge())
                    .expect("Reregistering connection failed!");
            }
        }

        optself
    }
}

impl InnerIoMachine<Connection> {
    fn parse_frames(mut self) -> Option<Self> {
        // Parse as many frames as we can.
        loop {
            if let Some(current) = self.io.current {
                debug!("FrameHeader already parsed, trying to parse frame.");
                let frame = Frame::parse(current,
                                         self.io.buffer.slice()
                                             .slice_from(FRAME_HEADER_LENGTH));

                match frame {
                    Err(parser::Error::Incomplete) => {
                        debug!("Not a full frame was parsed, tried to parse {:?} bytes",
                               self.io.buffer.len());
                        return Some(self)
                    },
                    Err(e) => {
                        debug!("Error parsing frame: {:?}", e);
                        return None;
                    },
                    Ok(frame) => {
                        debug!("Succesfully parsed frame {:?}", frame);

                        // Send the frame.
                        if let Err(e) = self.io.http2.apply(frame) {
                            error!("Http2 error: {:?}", e);
                            return None
                        }

                        // Recycle self.io.current and self.io.buffer.
                        self.io.current = None;

                        debug!("Creating a new buffer to replace with old.");
                        let mut newbuffer = buf();
                        newbuffer.fill(
                            &self.io.buffer[FRAME_HEADER_LENGTH + current.length as usize..]);

                        trace!("newbuffer contents = {:?}", newbuffer.slice());
                        self.io.buffer = newbuffer;
                    }
                }
            } else {
                debug!("No frame header parsed yet.");
                let header = FrameHeader::parse(&self.io.buffer);

                match header {
                    Err(::http2parse::Error::Short) => {
                        debug!("Not enough bytes for FrameHeader: {:?} bytes",
                               self.io.buffer.len());
                        return Some(self)
                    },
                    Err(e) => {
                        debug!("Error parsing frame header {:?}", e);
                        return None
                    },
                    Ok(header) => {
                        debug!("Parsed header: {:?}.", header);
                        self.io.current = Some(header);
                    }
                }
            }
        }
    }

    fn readable(mut self, event_loop: &mut EventLoop<LoopHandler>,
                handler: &mut LoopHandler) -> Option<Self> {
        debug!("Connection responding to readable event.");

        // Read in as much data as we can.
        loop {
            debug!("Reading from connection");
            match self.io.connection.try_read(self.io.buffer.get_write_buf()) {
                Ok(Some(0)) => {
                    debug!("Received EOF on Connection, deregistering token {:?}.",
                           self.token);
                    handler.deregister(&self, event_loop);
                    self.parse_frames();
                    return None
                },
                Ok(Some(n)) => {
                    debug!("Read {} bytes into buffer.", n);
                    unsafe { self.io.buffer.advance(n) }
                },
                Ok(None) => {
                    debug!("Read would block, yielding to parsing.");
                    return self.parse_frames()
                },
                Err(e) => {
                    error!("Connection read error {:?}", e);
                    handler.deregister(&self, event_loop);
                    return None
                }
            }
        }
    }

    fn writable(mut self, event_loop: &mut EventLoop<LoopHandler>,
                handler: &mut LoopHandler) -> Option<Self> {
        debug!("Connection responding to writable event.");

        'writable: loop {
            if let Some((mut encoder, cb)) = self.io.outgoing.take() {
                debug!("Popped frame encoder from outgoing.");
                trace!("FrameEncoder: {:?}", encoder);

                'encoder: loop {
                    match encoder.encode(&mut self.io.connection) {
                        EncodeResult::Wrote(n) => {
                            debug!("Wrote {} bytes from frame.", n);
                            continue 'encoder;
                        },
                        EncodeResult::WouldBlock(_) => {
                            debug!("Write would block, yielding.");
                            self.io.outgoing = Some((encoder, cb));
                            return Some(self);
                        },
                        EncodeResult::Error(e) => {
                            error!("Connection write error {:?}", e);
                            handler.deregister(&self, event_loop);
                            return None;
                        },
                        EncodeResult::Finished => {
                            debug!("Finished writing frame encoder.");
                            cb.0.call_box((&mut self.io.http2,));
                            break 'encoder;
                        },
                        EncodeResult::Eof => {
                            debug!("Received EOF on Connection, deregistering token {:?}.",
                                   self.token);
                            handler.deregister(&self, event_loop);
                            return None;
                        }
                    }
                }
            } else {
                match self.io.http2.outgoing.dequeue() {
                    Some(e) => {
                        debug!("Pulling next encoder.");
                        self.io.outgoing = Some(e);
                    }
                    None => {
                        debug!("No encoders available, yielding writable.");
                        return Some(self)
                    }
                }
            }
        }
    }
}

fn buf() -> AppendBuf {
    AppendBuf::new(FRAME_HEADER_LENGTH + FRAME_PAYLOAD_MAX_LENGTH)
}

