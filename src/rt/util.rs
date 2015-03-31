use mio::Evented;
use std::os::unix::io::{Fd, AsRawFd};

/// Convenience struct for internally handing
/// raw file descriptors over to the event loop.
#[derive(Debug, Copy)]
pub struct RawFd(Fd);

impl AsRawFd for RawFd {
    fn as_raw_fd(&self) -> Fd { self.0 }
}

impl Evented for RawFd { }

