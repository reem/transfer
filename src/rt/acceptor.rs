use std::sync::Arc;

use mio::{NonBlock, EventLoop, ReadHint, Token};
use mio::tcp::TcpListener;

use rt::loophandler::{LoopHandler, Registration};
use rt::connection::Connection;
use rt::Metadata;

use Handler as HttpHandler;

pub struct Acceptor {
    listener: NonBlock<TcpListener>,
    handler: Arc<Box<HttpHandler>>,
    metadata: Metadata
}

impl Acceptor {
    pub fn new(listener: NonBlock<TcpListener>,
               handler: Arc<Box<HttpHandler>>,
               metadata: Metadata) -> Acceptor {
        Acceptor {
            listener: listener,
            handler: handler,
            metadata: metadata
        }
    }

    pub fn readable(mut handler: &mut LoopHandler,
                    event_loop: &mut EventLoop<LoopHandler>,
                    token: Token, hint: ReadHint) {
        let (connection, httphandler, metadata) = {
            if let &mut Registration::Acceptor(ref mut acceptor) = &mut handler.slab[token] {
                (acceptor.listener.accept(),
                 acceptor.handler.clone(),
                 acceptor.metadata.clone())
            } else {
                unsafe { debug_unreachable!("LoopHandler yielded connection to acceptor.") }
            }
        };

        match connection {
            Ok(Some(connection)) => {
                let conn = Connection::new(connection, httphandler, metadata);
                handler.register(Registration::Connection(conn));
            },

            Ok(None) => {
                panic!("Incorrect readable hint.");
            }

            Err(_) => { }
        };
    }

    pub fn writable(handler: &mut LoopHandler,
                    event_loop: &mut EventLoop<LoopHandler>, token: Token) {
        unsafe { debug_unreachable!("Received writable hint on an acceptor.") }
    }
}

