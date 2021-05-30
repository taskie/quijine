use crate::Data;
use std::{error, fmt, io};

pub enum ErrorValue<'q> {
    None,
    String(String),
    Value(Data<'q>),
}

impl fmt::Display for ErrorValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        let default = "some error occured";
        match self {
            ErrorValue::None => f.write_str(default),
            ErrorValue::String(s) => f.write_str(s),
            ErrorValue::Value(v) => {
                let s = v.to_string().unwrap_or_else(|| default.into());
                f.write_str(s.as_str())
            }
        }
    }
}

impl fmt::Debug for ErrorValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

pub struct Error<'q> {
    pub value: ErrorValue<'q>,
}

impl<'q> Error<'q> {
    pub fn with_str<T: AsRef<str>>(message: T) -> Error<'static> {
        Error {
            value: ErrorValue::String(message.as_ref().to_string()),
        }
    }

    pub fn with_value(val: Data<'q>) -> Error<'q> {
        Error {
            value: ErrorValue::Value(val),
        }
    }
}

pub type Result<'q, T> = std::result::Result<T, Error<'q>>;

impl<'q> error::Error for Error<'q> {}

impl std::convert::From<Error<'_>> for io::Error {
    fn from(err: Error) -> Self {
        io::Error::new(io::ErrorKind::Other, format!("error: {:?}", err))
    }
}

impl fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        self.value.fmt(f)
    }
}

impl fmt::Debug for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}
