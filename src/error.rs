use std::{io, fmt};
use std::error::FromError;
use std::error::Error as StdError;

use mio::NotifyError;
use syncbox::util::async::AsyncError;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    NotifyQueueFull(::rt::Message),
    Async(AsyncError<()>),
    Executor
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref io) =>
                write!(fmt, "Falcon Error: {}", io),
            Error::NotifyQueueFull(_) =>
                fmt.write_str("Falcon Error: Notify Queue Full"),
            Error::Async(ref async) =>
                match *async {
                    AsyncError::Aborted =>
                        fmt.write_str("Falcon Error: Asynchronous Action Aborted"),
                    AsyncError::Failed(()) =>
                        fmt.write_str("Falcon Error: Asynchronous Action Failed")
                },
            Error::Executor =>
                fmt.write_str("Falcon Error: Executor Error.")
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
            Error::NotifyQueueFull(_) => None,
            Error::Async(_) => None, // TODO: File in syncbox to implement Error
            Error::Executor => None
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

impl FromError<AsyncError<()>> for Error {
    fn from_error(err: AsyncError<()>) -> Error {
        Error::Async(err)
    }
}

