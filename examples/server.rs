extern crate transfer;
extern crate mio;
extern crate env_logger;

use transfer::{rt, Handler};
use mio::{EventLoopConfig};
use mio::tcp::TcpListener;

use std::net::SocketAddr;
use std::sync::Arc;
use std::str::FromStr;
use std::thread;

fn main() {
    env_logger::init().unwrap();

    let metadata = rt::Metadata {
        executor: Arc::new(Box::new(ThreadExecutor))
    };

    let handle = rt::start(EventLoopConfig::new(), metadata).unwrap();

    let listener =
        TcpListener::bind(&SocketAddr::from_str("127.0.0.1:3000").unwrap())
            .unwrap();

    handle.register(listener, Arc::new(Box::new(NoopHandler))).unwrap();

    handle.await().unwrap();
}

struct ThreadExecutor;

impl rt::Executor for ThreadExecutor {
    fn execute(&self, task: rt::Thunk<'static>) {
        thread::spawn(move || task());
    }
}

struct NoopHandler;

impl Handler for NoopHandler {
    fn handle(&self) {}
}

