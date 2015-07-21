use std::thunk::Thunk;

use mio::util::Slab;
use mio::{self, EventLoop, Token, EventSet};

use rt::connection::Connection;
use rt::acceptor::Acceptor;
use rt::{Message, Metadata};

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

    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, mut events: EventSet) {
        if events.is_readable() {
            events = events - EventSet::readable();

            match self.slab[token] {
                Registration::Connection(_) =>
                    Connection::readable(self, event_loop, token, is_empty(&events)),
                Registration::Acceptor(_) =>
                    Acceptor::readable(self, event_loop, token, is_empty(&events))
            }
        }

        if events.is_writable() {
            events = events - EventSet::writable();

            let res = match self.slab[token] {
                Registration::Connection(_) => true,
                Registration::Acceptor(_) => false
            };

            if res {
                Connection::writable(self, event_loop, token, is_empty(&events))
            } else {
                Acceptor::writable(self, event_loop, token, is_empty(&events))
            }
        }

        if events.is_error() {

        }

        if events.is_hup() {

        }

        #[inline(always)]
        fn is_empty(e: &EventSet) -> bool { *e == EventSet::none() }
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

