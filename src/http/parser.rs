use appendbuf::Slice;

use std::{slice, mem};

use util::TypedSlice;

pub use http2parse::{
    FrameHeader, Priority, SizeIncrement,
    ErrorCode, Setting, StreamIdentifier,
    Flag, Kind
};

#[derive(Debug, Clone)]
pub struct Frame {
    pub header: FrameHeader,
    pub payload: Payload
}

impl Frame {
    pub fn parse(header: FrameHeader, buf: Slice) -> Result<Frame> {
        let raw = try!(::http2parse::Frame::parse(header, &buf));

        Ok(Frame {
            header: raw.header,
            payload: Payload::convert(raw.payload, &buf)
        })
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    Parse(::http2parse::Error),
    Incomplete
}

impl From<::http2parse::Error> for Error {
    fn from(err: ::http2parse::Error) -> Error {
        match err {
            ::http2parse::Error::Short => Error::Incomplete,
            e => Error::Parse(e)
        }
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Payload {
    Data(Slice),
    Headers {
        priority: Option<Priority>,
        block: Slice
    },
    Priority(Priority),
    Reset(ErrorCode),
    Settings(TypedSlice<Setting>),
    PushPromise {
        promised: StreamIdentifier,
        block: Slice
    },
    Ping(u64),
    GoAway {
        last: StreamIdentifier,
        error: ErrorCode,
        data: Slice
    },
    WindowUpdate(SizeIncrement),
    Continuation(Slice),
    Unregistered
}

impl Payload {
    /// Retrieve the Priority from this payload, if one is present.
    ///
    /// Only Headers and Priority frames can contain a Priority.
    pub fn priority(&self) -> Option<Priority> {
        match *self {
            Payload::Headers { ref priority, .. } => priority.clone(),
            Payload::Priority(ref priority) => Some(priority.clone()),
            _ => None
        }
    }

    fn convert(raw: ::http2parse::Payload, buf: &Slice) -> Payload {
        use http2parse::Payload as Raw;

        match raw {
            Raw::Data { data } =>
                Payload::Data(unsafe { convert_slice(buf, data) }),
            Raw::Headers { priority, block } =>
                Payload::Headers {
                    priority: priority,
                    block: unsafe { convert_slice(buf, block) }
                },
            Raw::Priority(priority) => Payload::Priority(priority),
            Raw::Reset(error) => Payload::Reset(error),
            Raw::Settings(settings) => {
                let settings_buf = unsafe {
                    slice::from_raw_parts(
                        settings.as_ptr() as *const u8,
                        settings.len() * mem::size_of::<Setting>())
                };

                Payload::Settings(unsafe {
                    TypedSlice::new(convert_slice(buf, settings_buf))
                })
            },
            Raw::PushPromise { promised, block } =>
                Payload::PushPromise {
                     promised: promised,
                     block: unsafe { convert_slice(buf, block) }
                },
            Raw::Ping(opaque) => Payload::Ping(opaque),
            Raw::GoAway { last, error, data } =>
                Payload::GoAway {
                    last: last,
                    error: error,
                    data: unsafe { convert_slice(buf, data) }
                },
            Raw::WindowUpdate(sz) => Payload::WindowUpdate(sz),
            Raw::Continuation(data) =>
                Payload::Continuation(unsafe { convert_slice(buf, data) }),
            Raw::Unregistered(_) => Payload::Unregistered
        }
    }
}

/// Convert a slice from a given Slice into an Slice over the same region.
unsafe fn convert_slice<'a>(buf: &Slice, slice: &'a [u8]) -> Slice {
    let bufstart = buf.as_ptr() as usize;
    let slice_start = slice.as_ptr() as usize;

    let start_offset = slice_start - bufstart;
    let end_offset = start_offset + slice.len();

    buf.slice(start_offset, end_offset)
}

#[cfg(test)]
mod tests {
    use prelude::*;
    use super::convert_slice;

    use appendbuf::{AppendBuf, Slice};

    fn slice(buf: &str) -> Slice {
        let mut outbuf = AppendBuf::new(buf.len());
        outbuf.fill(buf.as_bytes());
        outbuf.slice()
    }

    #[test]
    fn test_convert_slice() {
        let buf = slice("hello world");
        let slice = &buf[2..];
        let converted = unsafe { convert_slice(&buf, slice) };
        assert_eq!(b"llo world", &*converted);
    }
}

