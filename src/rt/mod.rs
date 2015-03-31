use mio::{self, EventLoop, EventLoopConfig, NonBlock, Evented, PollOpt, Interest};
use mio::tcp::TcpListener;
use iobuf::Allocator;
use syncbox::util::Run;
use syncbox::util::async::{Async, Future};

use std::thunk::Thunk;
use std::time::duration::Duration;
use std::os::unix::io::AsRawFd;
use std::fmt;

use rt::handler::Handler as RtHandler;
use rt::loophandler::LoopHandler;
use rt::util::RawFd;

use {Result, Error};
use Handler as HttpHandler;

mod loophandler;
mod util;
mod handler;

pub struct Handle {
    channel: mio::Sender<Message>,
    shutdown: Future<(), Error>
}

pub enum Message {
    NextTick(Thunk<'static>),
    Listener(NonBlock<TcpListener>, Box<HttpHandler>),
    Io(RawFd, Box<RtHandler>, PollOpt, Interest),
    Shutdown
}

pub enum TimeoutMessage {
    Later(Thunk<'static>, Duration)
}

impl Handle {
    pub fn on_next_tick<F: FnOnce() + Send + 'static>(&self, cb: F) -> Result<()> {
        self.send(Message::NextTick(Thunk::new(cb)))
    }

    pub fn register(&self, listener: NonBlock<TcpListener>,
                    handler: Box<HttpHandler>) -> Result<()> {
        self.send(Message::Listener(listener, handler))
    }

    pub fn register_io<E, R>(&self, io: &E, handler: R, pollopt: PollOpt,
                             interest: Interest) -> Result<()>
    where E: Evented, R: RtHandler {
        self.send(Message::Io(RawFd(io.as_raw_fd()), box handler, pollopt, interest))
    }

    pub fn shutdown(self) -> Result<Future<(), Error>> {
        try!(self.send(Message::Shutdown));
        Ok(self.shutdown)
    }

    fn send(&self, message: Message) -> Result<()> {
        Ok(try!(self.channel.send(message)))
    }
}

pub fn create<R>(config: EventLoopConfig, allocator: Box<Allocator>,
                 executor: R) -> Result<Handle>
where R: Run + Send + Sync {
    let mut eloop: EventLoop<LoopHandler<R>> = try!(EventLoop::configured(config));
    let mut handler = LoopHandler::new(allocator, executor);
    let channel = eloop.channel();

    // Run the event loop on the executor
    let local_executor = handler.executor.clone();

    let on_shutdown = local_executor.invoke(move || {
        eloop.run(&mut handler).map_err(Error::Io)
    }).map_err(|_| Error::Executor)
      .and_then(|x| match x {
        Ok(()) => Future::of(()),
        Err(e) => Future::error(e),
    });

    Ok(Handle {
        channel: channel,
        shutdown: on_shutdown
    })
}

impl fmt::Debug for Message {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Message::NextTick(_) => fmt.write_str("Message::NextTick(..)"),
            Message::Listener(_, _) => fmt.write_str("Message::Listener(..)"),
            Message::Io(raw, _, poll, interest) =>
                write!(fmt, "Message::RawFd({:?}, {:?}, {:?}, ..)", raw, poll, interest),
            Message::Shutdown => fmt.write_str("Message::Shutdown")
        }
    }
}

impl fmt::Debug for TimeoutMessage {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TimeoutMessage::Later(_, time) =>
                write!(fmt, "TimeoutMessage::Later(.., {:?})", time)
        }
    }
}

impl fmt::Display for TimeoutMessage {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, fmt)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, fmt)
    }
}


