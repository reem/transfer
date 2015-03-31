use std::sync::Arc;
use std::thunk::Thunk;

use iobuf::Allocator;
use mio::util::Slab;
use mio::{self, EventLoop, Token, ReadHint};
use syncbox::util::Run;

use rt::connection::Connection;
use rt::acceptor::Acceptor;
use rt::Message;

pub struct LoopHandler<R: Run> {
    pub allocator: Box<Allocator>,
    pub executor: Arc<R>,
    pub slab: Slab<Registration>
}

impl<R> LoopHandler<R> where R: Run {
    pub fn new(allocator: Box<Allocator>, executor: R) -> LoopHandler<R> {
        LoopHandler {
            allocator: allocator,
            executor: Arc::new(executor),
            slab: Slab::new(32 * 1024)
        }
    }

    fn register(&mut self, registration: Registration) {
        match registration {
            Registration::Connection(conn) => { },
            Registration::Acceptor(acceptor) => { }
        }
    }
}

pub enum Registration {
    Connection(Connection),
    Acceptor(Acceptor),
}

impl<R> mio::Handler for LoopHandler<R> where R: Run {
    type Message = Message;
    type Timeout = Thunk<'static>;

    fn readable(&mut self, event_loop: &mut EventLoop<LoopHandler<R>>,
                token: Token, hint: ReadHint) {
        // If a fildes was removed, ignore any hints.
        if !self.slab.contains(token) { return }

        match self.slab[token] {
            Registration::Connection(_) =>
                Connection::readable(self, event_loop, token, hint),
            Registration::Acceptor(_) =>
                Acceptor::readable(self, event_loop, token, hint)
        }
    }

    fn writable(&mut self, event_loop: &mut EventLoop<LoopHandler<R>>,
                token: Token) {
        // If a fildes was removed, ignore any hints.
        if !self.slab.contains(token) { return }

        let res = match self.slab[token] {
            Registration::Connection(_) => true,
            Registration::Acceptor(_) => false
        };

        if res {
            Connection::writable(self, event_loop, token)
        } else {
            Acceptor::writable(self, event_loop, token)
        }
    }

    fn notify(&mut self, event_loop: &mut EventLoop<LoopHandler<R>>,
              message: Message) {
        match message {
            Message::NextTick(thunk) => thunk.invoke(()),
            Message::Listener(listener, handler) => {
                self.register(Registration::Acceptor(Acceptor::new(listener, handler)))
            },
            Message::Shutdown => event_loop.shutdown(),
            Message::Timeout(thunk, ms) => { let _ = event_loop.timeout_ms(thunk, ms); }
        }
    }

    fn timeout(&mut self, event_loop: &mut EventLoop<LoopHandler<R>>,
               thunk: Thunk<'static>) {
        thunk.invoke(())
    }
}

