use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::io::Error;

#[derive(Debug)]
pub enum IMError {
    NotEnoughData,
    ContentMaxLen,
    TcpStreamEOF,
    Io(io::Error),
}

impl Display for IMError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IMError::NotEnoughData => write!(f, "Not enough data"),
            IMError::ContentMaxLen => write!(f, "The message exceeds the maximum length limit"),
            IMError::TcpStreamEOF => write!(f, "EOF reached"),
            IMError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl From<io::Error> for IMError {
    fn from(e: Error) -> Self {
        IMError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, IMError>;
