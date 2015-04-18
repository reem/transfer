use httparse as parser;
use iobuf::{AROIobuf, RWIobuf};

use std::{raw, mem};

use prelude::*;

pub const MAX_HEADERS: usize = 256;

pub struct RawHeader(pub AROIobuf, pub AROIobuf);
pub struct RawMethod(pub AROIobuf);
pub struct RawPath(pub AROIobuf);

pub struct RawRequest {
    pub method: RawMethod,
    pub path: RawPath,
    pub headers: [RawHeader; MAX_HEADERS],
    pub num_headers: usize,
    pub head_size: usize
}

pub struct RawResponse {
    pub version: u8,
    pub code: u16,
    pub headers: [RawHeader; MAX_HEADERS],
    pub num_headers: usize,
    pub head_size: usize
}

pub enum Error {
    Parse(parser::Error),
    Incomplete
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl RawRequest {
    pub fn new(buf: AROIobuf) -> Result<RawRequest> {
        let mut headers = [parser::EMPTY_HEADER; MAX_HEADERS];

        let (method, path, num_headers, head_size) = {
            let mut parser_request = parser::Request::new(&mut headers);
            let bytes = unsafe { buf.as_window_slice() };

            match parser_request.parse(bytes) {
                Ok(parser::Status::Complete(head_size)) => {
                    // TODO: Implement
                    panic!("Unimplemented: convert slices to iobufs.")
                },
                Ok(parser::Status::Partial) => return Err(Error::Incomplete),
                Err(err) => return Err(Error::Parse(err))
            }
        };

        Ok(RawRequest {
            method: method,
            path: path,
            headers: unsafe { convert_headers(buf, headers) },
            num_headers: num_headers,
            head_size: head_size
        })
    }
}

impl RawResponse {
    pub fn parse(buf: AROIobuf) -> Result<RawResponse> {
        let mut headers = [parser::EMPTY_HEADER; MAX_HEADERS];

        let (version, code, num_headers, head_size) = {
            let mut parser_response = parser::Request::new(&mut headers);
            let bytes = unsafe { buf.as_window_slice() };

            match parser_response.parse(bytes) {
                Ok(parser::Status::Complete(head_size)) => {
                    // TODO: Implement
                    panic!("Unimplemented: convert slices to iobufs.")
                },
                Ok(parser::Status::Partial) => return Err(Error::Incomplete),
                Err(err) => return Err(Error::Parse(err))
            }
        };

        Ok(RawResponse {
            version: version,
            code: code,
            headers: unsafe { convert_headers(buf, headers) },
            num_headers: num_headers,
            head_size: head_size
        })
    }
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

unsafe fn convert_headers<'a>(buf: AROIobuf, headers: [parser::Header<'a>; MAX_HEADERS]) -> [RawHeader; MAX_HEADERS] {
    let mut outheaders = initialize_blank_headers();

    for (inheader, outheader) in headers.iter().zip(outheaders.iter_mut()) {
        *outheader = RawHeader(unsafe { convert_slice(&buf, inheader.name.as_bytes()) },
                               unsafe { convert_slice(&buf, inheader.value) });
    }

    outheaders
}

fn initialize_blank_headers() -> [RawHeader; MAX_HEADERS] {
    let mut headers: [RawHeader; MAX_HEADERS] = unsafe { mem::uninitialized() };

    {
        let headers_slice: &mut [RawHeader] = &mut headers;
        for uninit_header in headers_slice {
            *uninit_header = {
                let onebuf = RWIobuf::new(0).atomic_read_only().unwrap();
                let twobuf = RWIobuf::new(0).atomic_read_only().unwrap();
                RawHeader(onebuf, twobuf)
            };
        }
    }

    headers
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

