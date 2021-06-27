use crate::Value;
use std::{error::Error as StdError, fmt, result::Result as StdResult, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    EvalError,
    RangeError,
    ReferenceError,
    SyntaxError,
    TypeError,
    URIError,
    InternalError,
    AggregateError,
    ExternalError,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ErrorKind::EvalError => "EvalError",
            ErrorKind::RangeError => "RangeError",
            ErrorKind::ReferenceError => "ReferenceError",
            ErrorKind::SyntaxError => "SyntaxError",
            ErrorKind::TypeError => "TypeError",
            ErrorKind::URIError => "URIError",
            ErrorKind::InternalError => "InternalError",
            ErrorKind::AggregateError => "AggregateError",
            ErrorKind::ExternalError => "ExternalError",
        })
    }
}

#[derive(Debug)]
pub struct JsErrorData {
    pub name: Option<String>,
    pub message: Option<String>,
    pub file_name: Option<String>,
    pub line_number: Option<i32>,
    pub stack: Option<String>,
}

impl fmt::Display for JsErrorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        let empty = &"".to_owned();
        let name = self.name.as_ref().unwrap_or(empty);
        let message = self.message.as_ref().unwrap_or(empty);
        f.write_fmt(format_args!("{}: {}", name, message))?;
        if let Some(stack) = self.stack.as_ref() {
            f.write_fmt(format_args!("\n{}", stack))?;
        }
        Ok(())
    }
}

pub enum ErrorValue {
    None,
    String(String),
    JsError(JsErrorData),
    External(Arc<dyn StdError + Send + Sync>),
}

impl fmt::Display for ErrorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        let default = "";
        match self {
            ErrorValue::None => f.write_str(default),
            ErrorValue::String(s) => f.write_str(s),
            ErrorValue::JsError(e) => e.fmt(f),
            ErrorValue::External(e) => e.fmt(f),
        }
    }
}

impl fmt::Debug for ErrorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

pub struct Error {
    pub kind: ErrorKind,
    pub value: ErrorValue,
}

impl Error {
    pub fn with_str(kind: ErrorKind, message: &str) -> Error {
        Error {
            kind,
            value: ErrorValue::String(message.to_owned()),
        }
    }

    pub fn with_external<T: Into<Box<dyn StdError + Send + Sync>>>(kind: ErrorKind, external: T) -> Error {
        Error {
            kind,
            value: ErrorValue::External(external.into().into()),
        }
    }

    pub fn external<T: Into<Box<dyn StdError + Send + Sync>>>(external: T) -> Error {
        Error::with_external(ErrorKind::ExternalError, external)
    }

    pub fn from_value<'q, T: Into<Value<'q>>>(kind: ErrorKind, value: T) -> Error {
        let value: Value<'q> = value.into();
        let ctx = value.context();
        let json = ctx.json_stringify_into(value, ctx.undefined(), ctx.undefined());
        Error {
            kind,
            value: ErrorValue::String(match json {
                Ok(s) => s,
                Err(e) => format!(r##"{{"name":"SystemError","message":"can't convert error: {}"}}"##, e),
            }),
        }
    }

    pub fn from_js_error<'q, T: Into<Value<'q>>>(kind: ErrorKind, value: T) -> Error {
        let value: Value<'q> = value.into();
        let value = JsErrorData {
            name: value.get("name").ok(),
            message: value.get("message").ok(),
            file_name: value.get("fileName").ok(),
            line_number: value.get("lineNumber").ok(),
            stack: value.get("stack").ok(),
        };
        Error {
            kind,
            value: ErrorValue::JsError(value),
        }
    }
}

impl<'q> StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self.value {
            ErrorValue::External(ref e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl From<Box<dyn StdError + Send + Sync>> for Error {
    fn from(e: Box<dyn StdError + Send + Sync>) -> Self {
        Error::external(e)
    }
}

impl From<Box<dyn StdError>> for Error {
    fn from(e: Box<dyn StdError>) -> Self {
        Error::with_str(ErrorKind::ExternalError, &format!("{}", e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        f.write_fmt(format_args!("{}: {}", self.kind, self.value))
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> StdResult<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}
pub trait ExternalError {
    fn to_qj_err(self) -> Error;
}

impl<E> ExternalError for E
where
    E: Into<Box<dyn StdError + Send + Sync>>,
{
    fn to_qj_err(self) -> Error {
        Error::external(self)
    }
}
