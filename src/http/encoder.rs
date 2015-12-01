use appendbuf::Slice;

use http::parser::{Frame, Priority, Payload, FrameHeader};
use std::io;

pub trait Encoder {
    fn encode<W: io::Write>(&mut self, write: &mut W) -> EncodeResult;
}

#[derive(Debug)]
pub enum EncodeResult {
    Wrote(usize),
    Finished,
    WouldBlock(usize),
    Eof,
    Error(io::Error)
}

impl EncodeResult {
    fn from_bytes(n: usize) -> EncodeResult {
        if n == 0 {
            EncodeResult::Finished
        } else {
            EncodeResult::Wrote(n)
        }
    }
}

macro_rules! try_encode {
    ($e:expr, $previous:expr) => {
        match $e {
            EncodeResult::Wrote(n) => n,
            EncodeResult::Finished => 0,
            EncodeResult::WouldBlock(n) => return EncodeResult::WouldBlock(n + $previous),
            e => return e
        }
    }
}


#[derive(Debug, Clone)]
pub struct FrameEncoder {
    header: FrameHeaderEncoder,
    payload: PayloadEncoder
}

impl From<Frame> for FrameEncoder {
    fn from(frame: Frame) -> FrameEncoder {
        FrameEncoder {
            header: FrameHeaderEncoder::from(frame.header),
            payload: PayloadEncoder::from(frame.payload)
        }
    }
}

impl Encoder for FrameEncoder {
    fn encode<W: io::Write>(&mut self, write: &mut W) -> EncodeResult {
        let n = try_encode!(self.header.encode(write), 0);
        let m = try_encode!(self.payload.encode(write), n);
        EncodeResult::from_bytes(n + m)
    }
}

#[derive(Debug, Clone)]
enum PayloadEncoder {
    Data(SliceEncoder),
    Headers {
        priority: PriorityEncoder,
        block: SliceEncoder
    },
    Priority(PriorityEncoder),
    Reset(U32Encoder),
    Settings(SliceEncoder),
    PushPromise {
        promised: U32Encoder,
        block: SliceEncoder
    },
    Ping(U64Encoder),
    GoAway {
        last: U32Encoder,
        error: U32Encoder,
        data: SliceEncoder
    },
    WindowUpdate(U32Encoder),
    Continuation(SliceEncoder),
    Unregistered(SliceEncoder)
}

impl From<Payload> for PayloadEncoder {
    fn from(payload: Payload) -> PayloadEncoder {
        use self::PayloadEncoder::*;

        match payload {
            Payload::Data(slice) => Data(SliceEncoder::from(slice)),
            Payload::Headers { priority, block } => Headers {
                priority: PriorityEncoder::new(priority),
                block: SliceEncoder::from(block)
            },
            Payload::Priority(priority) =>
                Priority(PriorityEncoder::new(Some(priority))),
            Payload::Reset(data) => Reset(U32Encoder::from(data.0)),
            Payload::Settings(settings) =>
                Settings(SliceEncoder::from(settings.into_slice())),
            Payload::PushPromise { promised, block } => PushPromise {
                 promised: U32Encoder::from(promised.0),
                 block: SliceEncoder::from(block)
            },
            Payload::Ping(data) => Ping(U64Encoder::from(data)),
            Payload::GoAway { last, error, data } => GoAway {
                last: U32Encoder::from(last.0),
                error: U32Encoder::from(error.0),
                data: SliceEncoder::from(data)
            },
            Payload::WindowUpdate(increment) =>
                WindowUpdate(U32Encoder::from(increment.0)),
            Payload::Continuation(block) => Continuation(SliceEncoder::from(block)),
            Payload::Unregistered(block) => Unregistered(SliceEncoder::from(block))
        }
    }
}

impl Encoder for PayloadEncoder {
    fn encode<W: io::Write>(&mut self, write: &mut W) -> EncodeResult {
        match *self {
            PayloadEncoder::Data(ref mut encoder) => encoder.encode(write),
            PayloadEncoder::Headers { ref mut priority, ref mut block } => {
                let n = try_encode!(priority.encode(write), 0);
                let m = try_encode!(block.encode(write), n);
                EncodeResult::from_bytes(n + m)
            },
            PayloadEncoder::Priority(ref mut priority) => priority.encode(write),
            PayloadEncoder::Reset(ref mut encoder) => encoder.encode(write),
            PayloadEncoder::Settings(ref mut settings) => settings.encode(write),
            PayloadEncoder::PushPromise { ref mut promised, ref mut block } => {
                let n = try_encode!(promised.encode(write), 0);
                let m = try_encode!(block.encode(write), n);
                EncodeResult::from_bytes(n + m)
            },
            PayloadEncoder::Ping(ref mut encoder) => encoder.encode(write),
            PayloadEncoder::GoAway { ref mut last, ref mut error, ref mut data } => {
                let n = try_encode!(last.encode(write), 0);
                let m = try_encode!(error.encode(write), n);
                let o = try_encode!(data.encode(write), n + m);
                EncodeResult::from_bytes(n + m + o)
            },
            PayloadEncoder::WindowUpdate(ref mut encoder) => encoder.encode(write),
            PayloadEncoder::Continuation(ref mut encoder) => encoder.encode(write),
            PayloadEncoder::Unregistered(ref mut encoder) => encoder.encode(write)
        }
    }
}

#[derive(Debug, Clone)]
struct SliceEncoder {
    slice: Slice,
    position: usize
}

impl From<Slice> for SliceEncoder {
    fn from(slice: Slice) -> SliceEncoder {
        SliceEncoder {
            slice: slice,
            position: 0
        }
    }
}

impl Encoder for SliceEncoder {
    fn encode<W: io::Write>(&mut self, write: &mut W) -> EncodeResult {
        if self.slice.len() == 0 || self.slice.len() == self.position {
            return EncodeResult::Finished
        }

        match write.write(&self.slice[self.position..]) {
            Ok(0) => EncodeResult::Eof,
            Ok(n) => {
                self.position += n;
                EncodeResult::Wrote(n)
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock =>
                EncodeResult::WouldBlock(0),
            Err(e) => EncodeResult::Error(e)
        }
    }
}

impl From<FrameHeader> for FrameHeaderEncoder {
    fn from(header: FrameHeader) -> FrameHeaderEncoder {
        let mut buffer = [0; 9];
        header.encode(&mut buffer);

        FrameHeaderEncoder {
            buffer: buffer,
            position: 0
        }
    }
}

impl From<u32> for U32Encoder {
    fn from(val: u32) -> U32Encoder {
        use ::byteorder::ByteOrder;

        let mut buffer = [0; 4];
        ::byteorder::BigEndian::write_u32(&mut buffer, val);

        U32Encoder {
            buffer: buffer,
            position: 0
        }
    }
}

impl From<u64> for U64Encoder {
    fn from(val: u64) -> U64Encoder {
        use ::byteorder::ByteOrder;

        let mut buffer = [0; 8];
        ::byteorder::BigEndian::write_u64(&mut buffer, val);

        U64Encoder {
            buffer: buffer,
            position: 0
        }
    }
}

impl PriorityEncoder {
    fn new(priority: Option<Priority>) -> Self {
        match priority {
            None => PriorityEncoder {
                buffer: [0; 5],
                position: 5
            },
            Some(priority) => {
                let mut buffer = [0; 5];
                priority.encode(&mut buffer);

                PriorityEncoder {
                     buffer: buffer,
                     position: 0
                }
            }
        }
    }
}

macro_rules! small_buffer_encoder {
    ($name:ident, $buffer_size:expr) => {
        #[derive(Debug, Clone)]
        struct $name {
            buffer: [u8; $buffer_size],
            position: u8
        }

        impl Encoder for $name {
            fn encode<W: io::Write>(&mut self, write: &mut W) -> EncodeResult {
                if self.position == $buffer_size {
                    return EncodeResult::Finished;
                }

                match write.write(&self.buffer[self.position as usize..]) {
                    Ok(0) => EncodeResult::Eof,
                    Ok(n) => {
                        self.position += n as u8;
                        EncodeResult::Wrote(n)
                    },
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock =>
                        EncodeResult::WouldBlock(0),
                    Err(e) => EncodeResult::Error(e)
                }
            }
        }
    }
}

small_buffer_encoder! { FrameHeaderEncoder, 9 }
small_buffer_encoder! { PriorityEncoder, 5 }
small_buffer_encoder! { U64Encoder, 8 }
small_buffer_encoder! { U32Encoder, 4 }

#[test]
fn test_slice_encoder() {
    use ::appendbuf::AppendBuf;

    let mut abuf = AppendBuf::new(10);
    abuf.fill(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    let mut result = vec![0; 10];
    let mut one_byte_slice = SliceEncoder::from(abuf.slice().slice_to(1));
    match one_byte_slice.encode(&mut &mut *result) {
        EncodeResult::Wrote(1) => {},
        e => panic!("Bad encode result {:?}, expected {:?}",
                    e, EncodeResult::Wrote(1))
    };
    assert_eq!(&result[..1], &[1]);

    let mut result = vec![0; 10];
    let mut empty_slice = SliceEncoder::from(abuf.slice().slice_to(0));
    match empty_slice.encode(&mut &mut *result) {
        EncodeResult::Finished => {},
        e => panic!("Bad encode result {:?}, expected {:?}",
                    e, EncodeResult::Finished)
    };
    assert_eq!(&*result, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}

#[cfg(all(test, feature = "random"))]
mod test {
    use http::encoder::{FrameEncoder, Encoder, EncodeResult};
    use http::parser::Frame;
    use http2parse::Frame as RawFrame;

    use std::io;

    /// An io::Write instance which alternates accepting one byte
    /// and returning WouldBlock, for testing.
    struct Stutter {
        // If false, will return WouldBlock next.
        active: bool,
        buffer: io::Cursor<Vec<u8>>
    }

    impl Stutter {
        fn new(size: usize) -> Stutter {
            Stutter {
                active: true,
                buffer: io::Cursor::new(vec![0; size])
            }
        }
    }

    impl io::Write for Stutter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let active = self.active;
            self.active = !active;

            if active {
                self.buffer.write(&buf[..1])
            } else {
                Err(io::Error::new(io::ErrorKind::WouldBlock, ""))
            }
        }

        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }

    #[test]
    fn test_frame_encoder() {
        fn check(raw_frame: RawFrame) {
            let mut encoder = FrameEncoder::from(Frame::clone_from(raw_frame));

            let mut raw_encode_buf = vec![0; raw_frame.encoded_len()];
            raw_frame.encode(&mut raw_encode_buf);

            let mut stuttered = Stutter::new(raw_frame.encoded_len());
            for i in 0..raw_frame.encoded_len() {
                loop {
                    match encoder.encode(&mut stuttered) {
                        EncodeResult::WouldBlock(0) |
                        EncodeResult::WouldBlock(1) |
                        EncodeResult::Finished => break,

                        EncodeResult::Wrote(1) => continue,

                        e => panic!("Bad encode result {:?}", e)
                    }
                }

                // Make sure they encode the same.
                let encoded = &stuttered.buffer.get_ref()[..i];
                let raw = &raw_encode_buf[..i];
                if encoded != raw {
                    panic!("Assertion error encoding {:#?}, {:#?} != {:#?} at {:?} with encoder {:#?}",
                           raw_frame, raw, encoded, i, encoder);
                }
            }
        }

        for _ in 0..1000 {
            check(::rand::random())
        }
    }
}

