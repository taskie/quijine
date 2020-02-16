use crate::{types::QjAny, Qj};
use bitflags::_core::fmt::{Error, Formatter};
use std::fmt;

pub enum QjErrValue<'q> {
    None,
    String(String),
    Value(Qj<'q, QjAny>),
}

impl fmt::Display for QjErrValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let default = "some error occured";
        match self {
            QjErrValue::None => f.write_str(default),
            QjErrValue::String(s) => f.write_str(s),
            QjErrValue::Value(v) => {
                let s = v.to_string().unwrap_or(default.into());
                f.write_str(s.as_str())
            }
        }
    }
}

impl fmt::Debug for QjErrValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        fmt::Display::fmt(self, f)
    }
}

pub struct QjErr<'q> {
    pub value: QjErrValue<'q>,
}

impl<'q> QjErr<'q> {
    pub fn from_str<T: AsRef<str>>(message: T) -> QjErr<'static> {
        QjErr {
            value: QjErrValue::String(message.as_ref().to_string()),
        }
    }

    pub fn from_value(val: Qj<'q, QjAny>) -> QjErr<'q> {
        QjErr {
            value: QjErrValue::Value(val),
        }
    }
}

pub type QjResult<'q, T> = std::result::Result<T, QjErr<'q>>;

impl std::convert::From<QjErr<'_>> for std::io::Error {
    fn from(err: QjErr) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, format!("error: {:?}", err))
    }
}

impl fmt::Display for QjErr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.value.fmt(f)
    }
}

impl fmt::Debug for QjErr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        fmt::Display::fmt(self, f)
    }
}
