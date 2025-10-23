/// Crate errors

/// Error
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    /// No device found
    NoDevice,
    /// Failed to lock device
    DeviceLock,
    /// Invalid address
    Address,
    /// Parse error
    Parse,
    /// Value out of bounds
    OutOfBound,
    /// Item not found
    NotFound,
    // I/O error
     Io(std::io::ErrorKind),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error.kind())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_: std::num::ParseIntError) -> Self {
        Error::Parse
    }
}
