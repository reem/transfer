use super::Thunk;

use mio::util::Slab;
use mio::{self, EventLoop, Token, EventSet, Evented, PollOpt};

use rt::connection::Connection;
use rt::acceptor::Acceptor;
use rt::{Message, Metadata};

use std::os::unix::io::AsRawFd;
use std::{io, mem};

pub struct LoopHandler {
    pub metadata: Metadata,
    pub slab: Slab<IoMachine>,
}

impl LoopHandler {
    pub fn new(metadata: Metadata) -> LoopHandler {
        LoopHandler {
            metadata: metadata,
            slab: Slab::new(32 * 1024)
        }
    }

    pub fn register<E: Evented>(&mut self, io: E, event_loop: &mut EventLoop<Self>,
                                interest: EventSet) -> Token
    where InnerIoMachine<E>: Into<IoMachine> {
        self.slab.insert_with(move |token| {
            let machine = InnerIoMachine { io: io, token: token };
            event_loop.register(&machine.io, token, interest, PollOpt::edge());
            machine.into()
        }).unwrap()
    }
}

pub trait EventMachine: Sized {
    fn ready(self, _: &mut EventLoop<LoopHandler>, _: &mut LoopHandler,
             _: EventSet) -> Option<Self> {
        Some(self)
    }
}

pub enum IoMachine {
    Connection(InnerIoMachine<Connection>),
    Acceptor(InnerIoMachine<Acceptor>),
    Active // The active IoMachine appears in the slab as Active
}

impl EventMachine for IoMachine {
    fn ready(self, event_loop: &mut EventLoop<LoopHandler>, handler: &mut LoopHandler,
             events: EventSet) -> Option<Self> {
        match self {
            IoMachine::Connection(machine) =>
                machine.ready(event_loop, handler, events).map(Into::into),
            IoMachine::Acceptor(machine) =>
                machine.ready(event_loop, handler, events).map(Into::into),
            IoMachine::Active =>
                panic!("Recursive readiness! IoMachine::ready called on Active.")
        }
    }
}

pub struct InnerIoMachine<I> {
    pub io: I,
    pub token: Token
}

impl Into<IoMachine> for InnerIoMachine<Connection> {
     fn into(self) -> IoMachine { IoMachine::Connection(self) }
}

impl Into<IoMachine> for InnerIoMachine<Acceptor> {
     fn into(self) -> IoMachine { IoMachine::Acceptor(self) }
}

fn with_io<I, F, T>(io_obj: &I, cb: F) -> T
where I: AsRawFd, F: FnOnce(&mio::Io) -> T {
    let io = mio::Io::from_raw_fd(io_obj.as_raw_fd());
    let val = cb(&io);
    mem::forget(io);
    val
}

impl mio::Evented for Connection {
     fn register(&self, selector: &mut mio::Selector, token: mio::Token,
                 interest: mio::EventSet, opts: mio::PollOpt) -> io::Result<()> {
         with_io(&self.connection, move |io| io.register(selector, token, interest, opts))
     }

     fn reregister(&self, selector: &mut mio::Selector, token: mio::Token,
                   interest: mio::EventSet, opts: mio::PollOpt) -> io::Result<()> {
         with_io(&self.connection, move |io| io.reregister(selector, token, interest, opts))
     }

     fn deregister(&self, selector: &mut mio::Selector) -> io::Result<()> {
         with_io(&self.connection, move |io| io.deregister(selector))
     }
}

impl mio::Evented for Acceptor {
     fn register(&self, selector: &mut mio::Selector, token: mio::Token,
                 interest: mio::EventSet, opts: mio::PollOpt) -> io::Result<()> {
         with_io(&self.listener, move |io| io.register(selector, token, interest, opts))
     }

     fn reregister(&self, selector: &mut mio::Selector, token: mio::Token,
                   interest: mio::EventSet, opts: mio::PollOpt) -> io::Result<()> {
         with_io(&self.listener, move |io| io.reregister(selector, token, interest, opts))
     }

     fn deregister(&self, selector: &mut mio::Selector) -> io::Result<()> {
         with_io(&self.listener, move |io| io.deregister(selector))
     }
}

impl mio::Handler for LoopHandler {
    type Message = Message;
    type Timeout = Thunk<'static>;

    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, mut events: EventSet) {
        if events.is_readable() {
            events = events - EventSet::readable();

            let new_machine = self.slab.replace(token, IoMachine::Active)
                .and_then(|machine| { machine.ready(event_loop, self, events) });

            match new_machine {
                Some(machine) => self.slab.replace(token, machine),
                None => self.slab.remove(token)
            };
        }
    }

    fn notify(&mut self, event_loop: &mut EventLoop<LoopHandler>,
              message: Message) {
        match message {
            Message::NextTick(thunk) => thunk(),
            Message::Listener(listener, handler) => {
                let metadata = self.metadata.clone();
                self.register(Acceptor::new(listener, handler, metadata), event_loop,
                              EventSet::readable());
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

