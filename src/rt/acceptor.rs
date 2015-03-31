use std::sync::Arc;

use mio::{NonBlock, EventLoop, ReadHint, Token};
use mio::tcp::TcpListener;
use syncbox::util::Run;

use rt::loophandler::{LoopHandler, Registration};
use rt::connection::Connection;

use Handler as HttpHandler;

pub struct Acceptor {
    listener: NonBlock<TcpListener>,
    handler: Arc<Box<HttpHandler>>
}

impl Acceptor {
    pub fn new(listener: NonBlock<TcpListener>,
               handler: Box<HttpHandler>) -> Acceptor {
        Acceptor {
            listener: listener,
            handler: Arc::new(handler)
        }
    }

    pub fn readable<R: Run>(mut handler: &mut LoopHandler<R>,
                            event_loop: &mut EventLoop<LoopHandler<R>>,
                            token: Token, hint: ReadHint) {
        let (httphandler, connection) = {
            if let &mut Registration::Acceptor(ref mut acceptor) = &mut handler.slab[token] {
                (acceptor.handler.clone(), acceptor.listener.accept())
            } else {
                unsafe { debug_unreachable!("LoopHandler yielded connection to acceptor.") }
            }
        };

        match connection {
            Ok(Some(connection)) => {
                let conn = Connection::new(connection, httphandler);
                handler.register(Registration::Connection(conn));
            },

            Ok(None) => {
                unsafe { debug_unreachable!("Incorrect readable hint.") }
            }

            Err(_) => { }
        };
    }

    pub fn writable<R: Run>(handler: &mut LoopHandler<R>,
                            event_loop: &mut EventLoop<LoopHandler<R>>,
                            token: Token) {
        unsafe { debug_unreachable!("Received writable hint on an acceptor.") }
    }
}

