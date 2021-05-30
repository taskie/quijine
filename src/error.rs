use crate::Data;
use std::{error, fmt, io};

pub enum QjErrorValue<'q> {
    None,
    String(String),
    Value(Data<'q>),
}

impl fmt::Display for QjErrorValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let default = "some error occured";
        match self {
            QjErrorValue::None => f.write_str(default),
            QjErrorValue::String(s) => f.write_str(s),
            QjErrorValue::Value(v) => {
                let s = v.to_string().unwrap_or_else(|| default.into());
                f.write_str(s.as_str())
            }
        }
    }
}

impl fmt::Debug for QjErrorValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

pub struct QjError<'q> {
    pub value: QjErrorValue<'q>,
}

impl<'q> QjError<'q> {
    pub fn with_str<T: AsRef<str>>(message: T) -> QjError<'static> {
        QjError {
            value: QjErrorValue::String(message.as_ref().to_string()),
        }
    }

    pub fn with_value(val: Data<'q>) -> QjError<'q> {
        QjError {
            value: QjErrorValue::Value(val),
        }
    }
}

pub type QjResult<'q, T> = std::result::Result<T, QjError<'q>>;

impl<'q> error::Error for QjError<'q> {}

impl std::convert::From<QjError<'_>> for io::Error {
    fn from(err: QjError) -> Self {
        io::Error::new(io::ErrorKind::Other, format!("error: {:?}", err))
    }
}

impl fmt::Display for QjError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.value.fmt(f)
    }
}

impl fmt::Debug for QjError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}
