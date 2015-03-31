use mio::{self, EventLoop, EventLoopConfig, NonBlock};
use mio::tcp::TcpListener;
use iobuf::Allocator;

use std::thunk::Thunk;
use std::sync::Arc;
use std::error::FromError;
use std::time::duration::Duration;
use std::fmt;

use rt::loophandler::LoopHandler;

use prelude::*;
use Handler as HttpHandler;

mod loophandler;
mod acceptor;
mod connection;

pub struct Handle {
    channel: mio::Sender<Message>,
    shutdown: Future<(), Error>
}

pub enum Message {
    NextTick(Thunk<'static>),
    Listener(NonBlock<TcpListener>, Box<HttpHandler>),
    Timeout(Thunk<'static>, u64),
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

    pub fn timeout_ms<F>(&self, cb: F, ms: u64) -> Result<()>
    where F: FnOnce() + Send + 'static {
        self.send(Message::Timeout(Thunk::new(cb), ms))
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
    let mut handler = LoopHandler::new(Arc::new(allocator), Arc::new(executor));
    let channel = eloop.channel();

    // Run the event loop on the executor
    let local_executor = handler.executor.clone();

    let on_shutdown = {
        let (tx, rx) = Future::pair();

        local_executor.run(move || {
            match eloop.run(&mut handler).map_err(FromError::from_error) {
                Ok(()) => tx.complete(()),
                Err(e) => tx.fail(e)
            }
        });

        rx
    };

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
            Message::Timeout(_, delay) =>
                write!(fmt, "Message::Timeout(.., {:?})", delay),
            Message::Shutdown => fmt.write_str("Message::Shutdown")
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, fmt)
    }
}


