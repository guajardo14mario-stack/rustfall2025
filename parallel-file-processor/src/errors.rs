use std::fmt;
use std::io;

#[derive(Debug, Clone)]
pub struct ProcessingError {
    pub message: String, // unified field for errors
}

#[derive(Debug, Clone)]
pub enum ProcessingErrorKind {
    Io(io::ErrorKind),
    Utf8,
    Other(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ProcessingError {}
