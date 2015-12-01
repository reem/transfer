use super::Thunk;

use mio::util::Slab;
use mio::{self, EventLoop, Token, EventSet, Evented, PollOpt};

use rt::connection::Connection;
use rt::acceptor::Acceptor;
use rt::{Message, Metadata};

use std::os::unix::io::AsRawFd;
use std::{io, mem, fmt};

#[derive(Debug)]
pub struct LoopHandler {
    pub metadata: Metadata,
    pub slab: Slab<LoopMachine>,
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
    where IoMachine<E>: Into<LoopMachine>, E: fmt::Debug {
        self.slab.insert_with(move |token| {
            let machine = IoMachine {
                io: io,
                token: token,
                interest: interest,
                pollopt: PollOpt::edge()
            };
            debug!("Registering new machine {:?} for token {:?}", machine, token);
            event_loop.register(&machine.io, machine.token,
                                machine.interest, machine.pollopt)
                .expect("Too many fds, cannot register more!");
            machine.into()
        }).unwrap()
    }

    pub fn deregister<E: Evented>(&mut self, io: &mut IoMachine<E>,
                                  event_loop: &mut EventLoop<Self>,
                                  interest: EventSet)
    where E: fmt::Debug {
        io.interest = io.interest - interest;

        if io.interest == EventSet::none() || io.interest == EventSet::hup() {
            if let Err(e) = event_loop.deregister(&io.io) {
                error!("Error when deregistering {:?} - {:?}", io, e);
            }
        } else {
            if let Err(e) = event_loop.reregister(&io.io, io.token,
                                                  io.interest, io.pollopt) {
                error!("Error when reregistering {:?} - {:?}", io, e)
            }
        }
    }
}

pub trait EventMachine: Sized {
    fn ready(self, _: &mut EventLoop<LoopHandler>, _: &mut LoopHandler,
             _: EventSet) -> Option<Self> {
        debug!("Default EventMachine implementation called.");
        Some(self)
    }
}

#[derive(Debug)]
pub enum LoopMachine {
    Connection(IoMachine<Connection>),
    Acceptor(IoMachine<Acceptor>),
    Active // The active LoopMachine appears in the slab as Active
}

impl EventMachine for LoopMachine {
    fn ready(self, event_loop: &mut EventLoop<LoopHandler>, handler: &mut LoopHandler,
             events: EventSet) -> Option<Self> {
        fn filter_no_interest<E>(io: IoMachine<E>) -> Option<IoMachine<E>> {
            if io.interest == EventSet::none() || io.interest == EventSet::hup() {
                None
            } else {
                Some(io)
            }
        }

        match self {
            LoopMachine::Connection(machine) =>
                machine.ready(event_loop, handler, events)
                    .and_then(filter_no_interest).map(Into::into),
            LoopMachine::Acceptor(machine) =>
                machine.ready(event_loop, handler, events)
                    .and_then(filter_no_interest).map(Into::into),
            LoopMachine::Active =>
                panic!("Recursive readiness! LoopMachine::ready called on Active.")
        }
    }
}

#[derive(Debug)]
pub struct IoMachine<I> {
    pub io: I,
    pub token: Token,
    pub interest: EventSet,
    pub pollopt: PollOpt
}

impl Into<LoopMachine> for IoMachine<Connection> {
    fn into(self) -> LoopMachine { LoopMachine::Connection(self) }
}

impl Into<LoopMachine> for IoMachine<Acceptor> {
    fn into(self) -> LoopMachine { LoopMachine::Acceptor(self) }
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

    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
        let old_machine = self.slab.replace(token, LoopMachine::Active);
        debug!("Event {:?} received on token {:?}, machine: {:?}", events, token, old_machine);

        let new_machine = old_machine
            .and_then(|machine| machine.ready(event_loop, self, events));

        match new_machine {
            Some(machine) => {
                debug!("New machine {:?} registered for token {:?}", machine, token);
                self.slab.replace(token, machine);
            },
            None => {
                debug!("Deregistering machine from slab with token {:?}", token);
                self.slab.remove(token);
            }
        };

        trace!("Finished processing event, slab: {:?}",
               self.slab.iter().collect::<Vec<_>>());
    }

    fn notify(&mut self, event_loop: &mut EventLoop<LoopHandler>,
              message: Message) {
        debug!("Notify message recieved: {:?}", message);
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

