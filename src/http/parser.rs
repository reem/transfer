use httparse as parser;
use iobuf::AROIobuf;

use prelude::*;

pub const MAX_HEADERS: usize = 256;

pub struct RawHeader(pub AROIobuf);
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
            headers: panic!("Unimplemented: convert headers to RawHeader."),
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
            headers: panic!("Unimplemented: convert headers to RawHeader."),
            num_headers: num_headers,
            head_size: head_size
        })
    }
}

