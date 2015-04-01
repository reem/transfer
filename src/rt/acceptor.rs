use std::sync::Arc;

use mio::{NonBlock, EventLoop, ReadHint, Token};
use mio::tcp::TcpListener;
use iobuf::Allocator;

use rt::loophandler::{LoopHandler, Registration};
use rt::connection::Connection;

use prelude::*;
use Handler as HttpHandler;

pub struct Acceptor {
    listener: NonBlock<TcpListener>,
    handler: Arc<Box<HttpHandler>>,
    allocator: Arc<Box<Allocator>>,
    executor: Arc<Box<Run + Send + Sync>>
}

impl Acceptor {
    pub fn new(listener: NonBlock<TcpListener>,
               handler: Arc<Box<HttpHandler>>,
               allocator: Arc<Box<Allocator>>,
               executor: Arc<Box<Run + Send + Sync>>) -> Acceptor {
        Acceptor {
            listener: listener,
            handler: handler,
            allocator: allocator,
            executor: executor
        }
    }

    pub fn readable(mut handler: &mut LoopHandler,
                    event_loop: &mut EventLoop<LoopHandler>,
                    token: Token, hint: ReadHint) {
        let (connection, httphandler, allocator, executor) = {
            if let &mut Registration::Acceptor(ref mut acceptor) = &mut handler.slab[token] {
                (acceptor.listener.accept(),
                 acceptor.handler.clone(),
                 acceptor.allocator.clone(),
                 acceptor.executor.clone())
            } else {
                unsafe { debug_unreachable!("LoopHandler yielded connection to acceptor.") }
            }
        };

        match connection {
            Ok(Some(connection)) => {
                let conn = Connection::new(connection, httphandler, allocator, executor);
                handler.register(Registration::Connection(conn));
            },

            Ok(None) => {
                unsafe { debug_unreachable!("Incorrect readable hint.") }
            }

            Err(_) => { }
        };
    }

    pub fn writable(handler: &mut LoopHandler,
                    event_loop: &mut EventLoop<LoopHandler>, token: Token) {
        unsafe { debug_unreachable!("Received writable hint on an acceptor.") }
    }
}

