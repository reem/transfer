use std::thunk::Thunk;

use mio::util::Slab;
use mio::{self, EventLoop, Token, ReadHint};

use rt::connection::Connection;
use rt::acceptor::Acceptor;
use rt::{Message, Executor, Metadata};

pub struct LoopHandler {
    pub metadata: Metadata,
    pub slab: Slab<Registration>
}

impl LoopHandler {
    pub fn new(metadata: Metadata) -> LoopHandler {
        LoopHandler {
            metadata: metadata,
            slab: Slab::new(32 * 1024)
        }
    }

    pub fn register(&mut self, registration: Registration) {
        // TODO: Fill in registration.
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

impl mio::Handler for LoopHandler {
    type Message = Message;
    type Timeout = Thunk<'static>;

    fn readable(&mut self, event_loop: &mut EventLoop<LoopHandler>,
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

    fn writable(&mut self, event_loop: &mut EventLoop<LoopHandler>,
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

    fn notify(&mut self, event_loop: &mut EventLoop<LoopHandler>,
              message: Message) {
        match message {
            Message::NextTick(thunk) => thunk(),
            Message::Listener(listener, handler) => {
                let metadata = self.metadata.clone();
                self.register(
                    Registration::Acceptor(
                        Acceptor::new(listener, handler, metadata)
                    )
                )
            },
            Message::Shutdown => event_loop.shutdown(),
            Message::Timeout(thunk, ms) => { let _ = event_loop.timeout_ms(thunk, ms); }
        }
    }

    fn timeout(&mut self, event_loop: &mut EventLoop<LoopHandler>,
               thunk: Thunk<'static>) {
        thunk()
    }
}

