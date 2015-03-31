use mio::{self, EventLoop, EventLoopConfig};
use iobuf::Allocator;
use syncbox::util::Run;
use syncbox::util::async::Future;

use self::loophandler::LoopHandler;
use {Result, Error};

mod loophandler;

pub struct Handle {
    channel: mio::Sender<Message>,
    at_exit: Future<Result<()>, ()>
}

pub enum Message {

}

pub enum TimeoutMessage {

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

