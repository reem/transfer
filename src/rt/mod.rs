use mio::{self, EventLoop, EventLoopConfig, NonBlock};
use mio::tcp::TcpListener;
use iobuf::Allocator;
use syncbox::util::Run;
use syncbox::util::async::Future;

use std::thunk::Thunk;
use std::time::duration::Duration;

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
    at_exit: Future<Result<()>, ()>
}

pub enum Message {
    NextTick(Thunk<'static>),
    Acceptor(NonBlock<TcpListener>, Box<HttpHandler>),
    Io(RawFd, Box<RtHandler>),
    Shutdown
}

pub enum TimeoutMessage {
    Later(Thunk<'static>, Duration)
}

impl Handle {

}

pub fn create<R>(config: EventLoopConfig, allocator: Box<Allocator>,
                 executor: R) -> Result<Handle>
where R: Run + Send + Sync {
    let mut eloop: EventLoop<LoopHandler<R>> = try!(EventLoop::configured(config));
    let mut handler = LoopHandler::new(allocator, executor);
    let channel = eloop.channel();

    // Run the event loop on the executor
    let local_executor = handler.executor.clone();
    let at_exit = local_executor.invoke(move || {
        eloop.run(&mut handler).map_err(Error::Io)
    });

    Ok(Handle {
        channel: channel,
        at_exit: at_exit
    })
}

