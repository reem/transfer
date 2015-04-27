use prelude::*;

pub mod parser;

pub struct Request {
    raw: parser::RawRequest
}

pub struct Response {
    raw: parser::RawResponse
}

impl From<parser::RawRequest> for Request {
    fn from(raw: parser::RawRequest) -> Request {
        Request { raw: raw }
    }
}

impl From<parser::RawResponse> for Response {
    fn from(raw: parser::RawResponse) -> Response {
        Response { raw: raw }
    }
}

