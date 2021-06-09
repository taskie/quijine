use quijine::{Error as QjError, ErrorKind};
use serde::ser;

#[derive(Debug)]
pub struct Error(QjError);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error(QjError::with_str(ErrorKind::InternalError, &format!("{}", msg)))
    }
}

impl From<QjError> for Error {
    fn from(e: QjError) -> Self {
        Error(e)
    }
}

impl From<Error> for QjError {
    fn from(e: Error) -> Self {
        e.0
    }
}

pub type Result<T> = std::result::Result<T, Error>;
