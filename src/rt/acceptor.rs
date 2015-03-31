use std::sync::Arc;

use mio::{NonBlock, EventLoop, ReadHint, Token};
use mio::tcp::TcpListener;
use syncbox::util::Run;

use rt::loophandler::LoopHandler;

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

    pub fn readable<R: Run>(handler: &mut LoopHandler<R>,
                            event_loop: &mut EventLoop<LoopHandler<R>>,
                            token: Token, hint: ReadHint) {

    }

    pub fn writable<R: Run>(handler: &mut LoopHandler<R>,
                            event_loop: &mut EventLoop<LoopHandler<R>>,
                            token: Token) {

    }
}

