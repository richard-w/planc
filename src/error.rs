use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    InvalidMessage,
    InsufficientPermissions,
    DuplicateName,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
