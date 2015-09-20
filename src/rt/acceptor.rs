use std::sync::Arc;

use mio::{EventLoop, Token, EventSet};
use mio::tcp::TcpListener;

use rt::loophandler::{LoopHandler, InnerIoMachine, EventMachine};
use rt::connection::Connection;
use rt::Metadata;

use Handler as HttpHandler;

pub struct Acceptor {
    pub listener: TcpListener,
    handler: Arc<Box<HttpHandler>>,
    metadata: Metadata
}

impl EventMachine for InnerIoMachine<Acceptor> {
    fn ready(self, event_loop: &mut EventLoop<LoopHandler>, handler: &mut LoopHandler,
             events: EventSet) -> Option<Self> {
        // Any other event is incorrect.
        assert_eq!(events, EventSet::readable());

        // Accept as many connections as possible.
        loop {
            let conn = match self.io.listener.accept() {
                Ok(Some(conn)) => conn,
                Ok(None) => break,
                Err(e) => {
                    error!("Acceptor error {:?}", e);
                    return None
                }
            };

            handler.register(
                Connection::new(conn.0, self.io.handler.clone(), self.io.metadata.clone()),
                event_loop, EventSet::readable() | EventSet::writable());
        }

        Some(self)
    }
}

impl Acceptor {
    pub fn new(listener: TcpListener,
               handler: Arc<Box<HttpHandler>>,
               metadata: Metadata) -> Acceptor {
        Acceptor {
            listener: listener,
            handler: handler,
            metadata: metadata
        }
    }
}

