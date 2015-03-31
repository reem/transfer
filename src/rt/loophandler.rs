use std::sync::Arc;

use iobuf::Allocator;
use mio::Handler;
use mio::util::Slab;
use syncbox::util::Run;

use rt::{Message, TimeoutMessage};

pub struct LoopHandler<R: Run> {
    pub allocator: Box<Allocator>,
    pub executor: Arc<R>,
    pub connections: Slab<()> // TODO: Replace with Connection
}

impl<R> LoopHandler<R> where R: Run {
    pub fn new(allocator: Box<Allocator>, executor: R) -> LoopHandler<R> {
        LoopHandler {
            allocator: allocator,
            executor: Arc::new(executor),
            connections: Slab::new(32 * 1024)
        }
    }
}

impl<R> Handler for LoopHandler<R> where R: Run {
    type Message = Message;
    type Timeout = TimeoutMessage;
}

