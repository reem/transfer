use Error;
use rt;

pub use self::stream::Stream;

pub mod parser;
pub mod stream;
pub mod error;

pub fn http(frames_rx: ::eventual::Stream<parser::Frame, Error>,
            metadata: rt::Metadata) {

}

