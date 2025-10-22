/// Crate errors

/// Error
#[derive(Debug)]
pub enum Error {
    /// No device found
    NoDevice,
    /// Failed to lock device
    DeviceLock,
    /// Invalid address
    Address,
    /// Underlying IO error
    Io(std::io::Error),
    /// Parse error
    Parse,
    /// Value out of bounds
    OutOfBound,
    /// Item not found
    NotFound,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_: std::num::ParseIntError) -> Self {
        Error::Parse
    }
}
