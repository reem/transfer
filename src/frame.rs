
const PAYLOAD_LENGTH_BITS: usize = 24;
const FRAME_TYPE_BITS: usize = 8;
const FRAME_FLAG_BITS: usize = 8;
const RESERVED_BIT: usize = 1;
const STREAM_IDENTIFIER_BITS: usize = 31;

const FRAME_HEADER_BITS: usize =
    PAYLOAD_LENGTH_BITS +
    FRAME_TYPE_BITS +
    FRAME_FLAG_BITS +
    RESERVED_BIT +
    STREAM_IDENTIFIER_BITS;

// From the spec, or FRAME_HEADER_BITS / 8
const FRAME_HEADER_BYTES: usize = 9;

pub struct Frame {
    length: u32,
    kind: Kind,
    flag: Flag,
    id: StreamIdentifier,
    payload: Payload
}

// contains type and flags
#[repr(u8)]
pub enum Kind {
    Data = 0,
    Headers = 1,
    Priority = 2,
    Reset = 3,
    Settings = 4,
    PushPromise = 5,
    Ping = 6,
    GoAway = 7,
    WindowUpdate = 8,
    Continuation = 9
}

// Bitflags?
#[repr(u8)]
pub enum Flag {
    EndStream = 0x1,
    Ack = 0x1,
    EndHeaders = 0x4,
    Padded = 0x8,
    Priority = 0x20
}

pub enum Payload<'a> {

}

// things that can go wrong during parsing
pub enum Error {
    /// A full frame header was not passed.
    Short
}

impl Frame {
    pub fn parse(buf: &[u8]) -> Result<Frame, Error> {
        let mut frame;
        if buf.len() < FRAME_FLAG_BYTES {
            frame.length = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | buf[2];
            frame.kind = buf[3] as Kind;
            frame.flag = buf[4] as Flag;
            frame.id = StreamIdentifier::parse(buf[5..]);

            // TODO: Parse payload
            Ok(frame)
        } else {
            Err(Error::Short)
        }
    }
}

pub struct StreamIdentifier(pub u32);

impl StreamIdentifier {
    pub fn parse(buf: &[u8]) -> StreamIdentifier {
        StreamIdentifier(
            ((buf[0] as u32) << 24) |
            ((buf[1] as u32) << 16) |
            ((buf[2] as u32) << 8) |
             (buf[3] as u32)
        )
    }
}

