use appendbuf::{AppendBuf, Slice};

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

impl<'a> PartialEq<::http2parse::Frame<'a>> for Frame {
    fn eq(&self, other: &::http2parse::Frame) -> bool {
        self.header == other.header &&
            self.payload == other.payload
    }
}

impl Frame {
    /// Convert a raw http2parse Frame into a transfer Frame by copying.
    ///
    /// The data within the frame is encoded then re-parsed into the new
    /// representation. This constructor is mostly for testing, and should
    /// not be used in performance-sensitive code.
    pub fn clone_from(frame: &::http2parse::Frame) -> Frame {
        let mut buf = AppendBuf::new(frame.encoded_len());

        frame.encode(buf.get_write_buf());
        unsafe { buf.advance(frame.encoded_len()); }

        Frame::parse(frame.header, buf.slice().slice_from(9)).unwrap()
    }

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
    Unregistered(Slice)
}

impl<'a> PartialEq<::http2parse::Payload<'a>> for Payload {
    fn eq(&self, other: &::http2parse::Payload) -> bool {
        use http2parse::Payload as P2;
        use self::Payload as P1;

        // ugh
        match (self, *other) {
            (&P1::Data(ref buf), P2::Data { data }) => &**buf == data,
            (&P1::Headers { priority: ref priority1, block: ref block1 },
             P2::Headers { ref priority, block }) =>
                priority1 == priority && &**block1 == block,
            (&P1::Priority(ref priority), P2::Priority(ref priority1)) =>
                priority == priority1,
            (&P1::Reset(err), P2::Reset(err1)) => err == err1,
            (&P1::Settings(ref settings), P2::Settings(settings1)) =>
                &**settings == settings1,
            (&P1::PushPromise { ref promised, ref block },
             P2::PushPromise { promised: ref promised1, block: block1 }) =>
                &**block == block1 && promised == promised1,
            (&P1::Ping(data), P2::Ping(data1)) => data == data1,
            (&P1::GoAway { ref last, error, ref data },
             P2::GoAway { last: ref last1, error: error1, data: data1 }) =>
                last == last && error == error1 && &**data == data1,
            (&P1::WindowUpdate(increment), P2::WindowUpdate(increment1)) =>
                increment == increment1,
            (&P1::Continuation(ref block), P2::Continuation(block1)) =>
                &**block == block1,
            (&P1::Unregistered(ref block), P2::Unregistered(block1)) =>
                &**block == block1,
            _ => false
        }
    }
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
            Raw::Unregistered(block) =>
                Payload::Unregistered(unsafe { convert_slice(buf, block) })
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
        let slice = &buf[2..4];
        let converted = unsafe { convert_slice(&buf, slice) };
        assert_eq!(b"ll", &*converted);
    }

    #[cfg(feature = "random")]
    mod rand {
        use appendbuf::AppendBuf;
        use http::parser::Frame;

        #[test]
        fn test_convert_frame() {
            fn roundtrip(frame: ::http2parse::Frame) {
                let new_frame = Frame::clone_from(&frame);
                assert_eq!(new_frame, frame);
            }

            for _ in 0..100 {
                roundtrip(::rand::random())
            }
        }

        #[test]
        fn test_frame_parse() {
            fn roundtrip(frame: ::http2parse::Frame) {
                let mut buf = AppendBuf::new(frame.encoded_len());
                frame.encode(buf.get_write_buf());
                unsafe { buf.advance(frame.encoded_len()) };
                assert_eq!(Frame::parse(frame.header,
                                        buf.slice().slice_from(9)).unwrap(),
                           frame);
            }

            for _ in 0..100 {
                roundtrip(::rand::random())
            }
        }
    }
}

