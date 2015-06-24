use iobuf::{AROIobuf, RWIobuf};

use std::{raw, mem};

use util::TypedAROIobuf;
use prelude::*;

pub use http2parse::{
    FrameHeader, Priority, SizeIncrement,
    ErrorCode, Setting, StreamIdentifier
};

pub struct Frame {
    pub header: FrameHeader,
    pub payload: Payload
}

pub enum Error {
    Parse(::http2parse::Error),
    Incomplete
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub enum Payload {
    Data(AROIobuf),
    Headers {
        priority: Option<Priority>,
        block: AROIobuf
    },
    Priority(Priority),
    Reset(ErrorCode),
    Settings(TypedAROIobuf<Setting>),
    PushPromise {
        promised: StreamIdentifier,
        block: AROIobuf
    },
    Ping(u64),
    GoAway {
        last: StreamIdentifier,
        error: ErrorCode,
        data: AROIobuf
    },
    WindowUpdate(SizeIncrement),
    Continuation(AROIobuf)
}

/// Convert a slice from a given AROIobuf into an AROIobuf over the same region.
unsafe fn convert_slice<'a>(buf: &AROIobuf, slice: &'a [u8]) -> AROIobuf {
    let bufstart = buf.as_window_slice().as_ptr() as u32;
    let raw::Slice { data, len } = mem::transmute::<&[u8], raw::Slice<u8>>(slice);

    let start_offset = (data as u32) - bufstart;
    let end_offset = start_offset + (len as u32);

    let mut outbuf = buf.clone();
    outbuf.sub_window(start_offset, end_offset);
    outbuf
}

#[cfg(test)]
mod tests {
    use prelude::*;
    use super::convert_slice;

    use iobuf::{AROIobuf, RWIobuf};

    fn aroiobuf(buf: &str) -> AROIobuf {
        RWIobuf::from_str_copy(buf).atomic_read_only().ok().unwrap()
    }

    #[test]
    fn test_slice_to_buf() {
        let abuf = aroiobuf("hello world");
        let slice = &unsafe { abuf.as_window_slice() }[3..];
        let converted = convert_slice(abuf, slice);
        assert_eq!(b"llo world", unsafe { converted.as_window_slice() });
    }
}

