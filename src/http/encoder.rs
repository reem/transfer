use appendbuf::Slice;

use http::parser::{Frame, Priority, Payload, FrameHeader};
use std::io;

pub trait Encoder {
    fn encode<W: io::Write>(&mut self, write: &mut W) -> Option<io::Result<usize>>;
}

/// Combinator for composing encoders.
fn try<F>(result: Option<io::Result<usize>>, cb: F, default: usize) -> Option<io::Result<usize>>
where F: FnOnce(usize) -> Option<io::Result<usize>> {
    match result {
        Some(Ok(0)) => Some(Ok(0)),
        Some(Ok(n)) => cb(n),
        None => cb(default),
        e => e
    }
}

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
    fn encode<W: io::Write>(&mut self, write: &mut W) -> Option<io::Result<usize>> {
        try(self.header.encode(write), |n| {
            try(self.payload.encode(write), |m| {
                Some(Ok(n + m))
            }, n)
        }, 0)
    }
}

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
    Unregistered
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
            Payload::Unregistered => Unregistered
        }
    }
}

impl Encoder for PayloadEncoder {
    fn encode<W: io::Write>(&mut self, write: &mut W) -> Option<io::Result<usize>> {
        match *self {
            PayloadEncoder::Data(ref mut encoder) => encoder.encode(write),
            PayloadEncoder::Headers { ref mut priority, ref mut block } =>
                try(priority.encode(write), |n| {
                    try(block.encode(write), |m| Some(Ok(n + m)), n)
                }, 0),
            PayloadEncoder::Priority(ref mut priority) => priority.encode(write),
            PayloadEncoder::Reset(ref mut encoder) => encoder.encode(write),
            PayloadEncoder::Settings(ref mut settings) => settings.encode(write),
            PayloadEncoder::PushPromise { ref mut promised, ref mut block } =>
                try(promised.encode(write), |n| {
                    try(block.encode(write), |m| Some(Ok(n + m)), n)
                }, 0),
            PayloadEncoder::Ping(ref mut encoder) => encoder.encode(write),
            PayloadEncoder::GoAway { ref mut last, ref mut error, ref mut data } =>
                try(last.encode(write), |n| {
                    try(error.encode(write), |m| {
                        try(data.encode(write), |o| Some(Ok(n + m + o)), n + m)
                    }, n)
                }, 0),
            PayloadEncoder::WindowUpdate(ref mut encoder) => encoder.encode(write),
            PayloadEncoder::Continuation(ref mut encoder) => encoder.encode(write),
            PayloadEncoder::Unregistered => None
        }
    }
}

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
    fn encode<W: io::Write>(&mut self, write: &mut W) -> Option<io::Result<usize>> {
        if self.slice.len() <= self.position { return None }

        match write.write(&self.slice[self.position..]) {
            Ok(n) => {
                self.position += n;
                Some(Ok(n))
            },
            e => Some(e)
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
        struct $name {
            buffer: [u8; $buffer_size],
            position: u8
        }

        impl Encoder for $name {
            fn encode<W: io::Write>(&mut self, write: &mut W) -> Option<io::Result<usize>> {
                if self.position >= $buffer_size {
                    return None
                }

                match write.write(&self.buffer[self.position as usize..]) {
                    Ok(n) => {
                        self.position += n as u8;
                        Some(Ok(n))
                    },
                    e => Some(e)
                }
            }
        }
    }
}

small_buffer_encoder! { FrameHeaderEncoder, 9 }
small_buffer_encoder! { PriorityEncoder, 5 }
small_buffer_encoder! { U64Encoder, 8 }
small_buffer_encoder! { U32Encoder, 4 }

