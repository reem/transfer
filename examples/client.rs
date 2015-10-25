extern crate http2parse;
extern crate rand;

use std::thread;

fn main() {
    let guards = (0..10).map(|_| { go() }).collect::<Vec<_>>();
    for guard in guards { guard.join().unwrap(); }
}

#[cfg(not(feature = "random"))]
fn go() -> thread::JoinHandle<()> {
    panic!("Running client without features = random.");
}

#[cfg(feature = "random")]
fn go() -> thread::JoinHandle<()> {
    use std::net::TcpStream;
    use std::io::Write;

    use http2parse::Frame;

    thread::spawn(move || {
        let mut stream = TcpStream::connect("localhost:3000").unwrap();

        let frame = ::rand::random::<Frame<'static>>();
        println!("Sending Frame: {:?}", frame);

        let mut buf = vec![0; 2000];
        let frame_len = frame.encode(&mut buf);

        stream.write(&buf[..frame_len]).unwrap();
    })
}
