use std::{io, fmt};
use std::error::FromError;
use std::error::Error as StdError;

use mio::NotifyError;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    NotifyQueueFull(::rt::Message)
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref io) => write!(fmt, "Falcon Error: {}", io),
            Error::NotifyQueueFull(_) => fmt.write_str("Falcon Error: Notify Queue Full.")
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        "Falcon Error"
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Io(ref io) => Some(io),
            Error::NotifyQueueFull(_) => None
        }
    }
}

impl FromError<io::Error> for Error {
    fn from_error(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl FromError<NotifyError<::rt::Message>> for Error {
    fn from_error(err: NotifyError<::rt::Message>) -> Error {
        match err {
            NotifyError::Io(io) => Error::Io(io),
            NotifyError::Full(m) => Error::NotifyQueueFull(m)
        }
    }
}

