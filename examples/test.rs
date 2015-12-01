#![feature(read_exact)]

extern crate http2parse;
extern crate rand;
extern crate transfer;
extern crate mio;
extern crate env_logger;
extern crate eventual;

use transfer::{rt, Handler};
use mio::{EventLoopConfig};
use mio::tcp::TcpListener;
use eventual::Async;

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

    let guards = (0..100).map(|_| { go() }).collect::<Vec<_>>();
    for guard in guards { guard.join().unwrap(); }

    handle.shutdown().unwrap().await().unwrap();
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

#[cfg(not(feature = "random"))]
fn go() -> thread::JoinHandle<()> {
    panic!("Running client without features = random.");
}

#[cfg(feature = "random")]
fn go() -> thread::JoinHandle<()> {
    use std::net::TcpStream;
    use std::io::{Read, Write};

    use http2parse::{Frame, FrameHeader};

    thread::spawn(move || {
        for _ in 0..100 {
            let mut stream = TcpStream::connect("localhost:3000").unwrap();

            for _ in 0..10 {
                let frame = ::rand::random::<Frame<'static>>();
                // println!("Sending Frame: {:?}", frame);

                let mut buf = vec![0; frame.encoded_len()];
                let frame_len = frame.encode(&mut buf);

                stream.write(&buf[..frame_len]).unwrap();

                buf = vec![0; frame.encoded_len()];
                stream.read_exact(&mut buf).unwrap();

                let returned_header = FrameHeader::parse(&buf[..9]).unwrap();
                let returned = Frame::parse(returned_header, &buf[9..]).unwrap();
                assert_eq!(frame, returned);
            }
        }
    })
}
