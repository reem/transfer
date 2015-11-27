#![feature(read_exact)]

extern crate http2parse;
extern crate rand;

use std::thread;

fn main() {
    let guards = (0..100).map(|_| { go() }).collect::<Vec<_>>();
    for guard in guards { guard.join().unwrap(); }
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
        for _ in 0..2 {
            let mut stream = TcpStream::connect("localhost:3000").unwrap();

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
    })
}
