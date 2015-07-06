use iobuf::{AROIobuf, RWIobuf};

use std::{raw, slice, mem};

use util::TypedAROIobuf;
use prelude::*;

pub use http2parse::{
    FrameHeader, Priority, SizeIncrement,
    ErrorCode, Setting, StreamIdentifier,
    ParserSettings
};

pub struct Frame {
    pub header: FrameHeader,
    pub payload: Payload
}

impl Frame {
    pub fn parse(header: FrameHeader, buf: AROIobuf,
                 settings: ParserSettings) -> Result<Frame> {
        let bytes = unsafe { buf.as_window_slice() };
        let raw = try!(::http2parse::Frame::parse(header, bytes, settings));

        Ok(Frame {
            header: raw.header,
            payload: Payload::convert(raw.payload, &buf)
        })
    }
}

pub enum Error {
    Parse(::http2parse::Error),
    Incomplete
}

impl From<::http2parse::Error> for Error {
    fn from(err: ::http2parse::Error) -> Error {
        Error::Parse(err)
    }
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
    Continuation(AROIobuf),
    Unregistered
}

impl Payload {
    fn convert(raw: ::http2parse::Payload, buf: &AROIobuf) -> Payload {
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
                    TypedAROIobuf::new(convert_slice(buf, settings_buf))
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
            Raw::Unregistered => Payload::Unregistered
        }
    }
}

/// Convert a slice from a given AROIobuf into an AROIobuf over the same region.
unsafe fn convert_slice<'a>(buf: &AROIobuf, slice: &'a [u8]) -> AROIobuf {
    let bufstart = buf.as_window_slice().as_ptr() as u32;
    let raw::Slice { data, len } = mem::transmute::<&[u8], raw::Slice<u8>>(slice);

    let start_offset = (data as u32) - bufstart;
    let end_offset = start_offset + (len as u32);

    let mut outbuf = buf.clone();
    outbuf.sub_window(start_offset, end_offset).unwrap();
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

