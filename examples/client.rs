extern crate http2parse;
extern crate rand;

use std::net::TcpStream;
use std::io::Write;
use std::thread;

use http2parse::Frame;

fn main() {
    let mut guards = vec![];

    for _ in 0..10 {
        let guard = thread::spawn(move || {
            let mut stream = TcpStream::connect("localhost:3000").unwrap();

            let frame = ::rand::random::<Frame<'static>>();
            println!("Sending Frame: {:?}", frame);

            let mut buf = vec![0; 2000];
            let frame_len = frame.encode(&mut buf);

            stream.write(&buf[..frame_len]).unwrap();
        });

        guards.push(guard);
    }

    for guard in guards { guard.join().unwrap(); }
}

