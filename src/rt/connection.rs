use mio::{NonBlock, EventLoop, ReadHint, Token};
use mio::tcp::TcpStream;
use syncbox::util::Run;

use rt::loophandler::LoopHandler;

pub struct Connection {
    stream: NonBlock<TcpStream>
}

impl Connection {
    pub fn readable<R: Run>(handler: &mut LoopHandler<R>,
                            event_loop: &mut EventLoop<LoopHandler<R>>,
                            token: Token, hint: ReadHint) {

    }

    pub fn writable<R: Run>(handler: &mut LoopHandler<R>,
                            event_loop: &mut EventLoop<LoopHandler<R>>,
                            token: Token) {

    }
}

