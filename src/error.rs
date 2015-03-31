use std::{io, fmt};
use std::error::FromError;
use std::error::Error as StdError;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Io(io::Error)
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref io) => write!(fmt, "Falcon Error: {}", io)
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        "Falcon Error"
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Io(ref io) => Some(io)
        }
    }
}

impl FromError<io::Error> for Error {
    fn from_error(err: io::Error) -> Error {
        Error::Io(err)
    }
}

