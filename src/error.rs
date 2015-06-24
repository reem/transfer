use std::{io, fmt};
use std::error::Error as StdError;

use mio::NotifyError;
use eventual::AsyncError;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    NotifyQueueFull(::rt::Message),
    NotifyQueueClosed,
    Async(AsyncError<()>),
    Executor
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref io) =>
                write!(fmt, "Transfer Error: {}", io),
            Error::NotifyQueueFull(_) =>
                fmt.write_str("Transfer Error: Notify Queue Full"),
            Error::Async(ref async) =>
                match *async {
                    AsyncError::Aborted =>
                        fmt.write_str("Transfer Error: Asynchronous Action Aborted"),
                    AsyncError::Failed(()) =>
                        fmt.write_str("Transfer Error: Asynchronous Action Failed")
                },
            Error::Executor =>
                fmt.write_str("Transfer Error: Executor Error."),
            Error::NotifyQueueClosed =>
                fmt.write_str("Transfer Error: Notify Queue is Closed")
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        "Transfer Error"
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Io(ref io) => Some(io),
            Error::NotifyQueueFull(_) => None,
            Error::Async(_) => None, // TODO: File in syncbox to implement Error
            Error::Executor => None,
            Error::NotifyQueueClosed => None
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<NotifyError<::rt::Message>> for Error {
    fn from(err: NotifyError<::rt::Message>) -> Error {
        match err {
            NotifyError::Io(io) => Error::Io(io),
            NotifyError::Full(m) => Error::NotifyQueueFull(m),
            NotifyError::Closed(_) => Error::NotifyQueueClosed
        }
    }
}

impl From<AsyncError<()>> for Error {
    fn from(err: AsyncError<()>) -> Error {
        Error::Async(err)
    }
}

