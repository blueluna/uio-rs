/// Crate errors

#[derive(Debug)]
pub enum Error {
    NoDevice,
    Address,
    Io(std::io::Error),
    Parse,
    OutOfBound,
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
