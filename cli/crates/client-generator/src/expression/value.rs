mod object;

pub use object::Object;
use std::fmt;

use crate::common::{Quoted, Template};

#[derive(Debug)]
pub struct Value {
    kind: ValueKind,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ValueKind::Object(ref obj) => obj.fmt(f),
            ValueKind::String(ref s) => s.fmt(f),
            ValueKind::Template(ref s) => s.fmt(f),
            ValueKind::Number(n) => {
                if n.is_nan() {
                    f.write_str("NaN")
                } else if n.is_infinite() && n.is_sign_positive() {
                    f.write_str("Infinity")
                } else if n.is_infinite() {
                    f.write_str("-Infinity")
                } else if n == n.trunc() {
                    write!(f, "{n:.0}")
                } else {
                    n.fmt(f)
                }
            }
            ValueKind::Boolean(b) => b.fmt(f),
            ValueKind::Null => f.write_str("null"),
            ValueKind::Undefined => f.write_str("undefined"),
        }
    }
}

impl<T> From<T> for Value
where
    T: Into<ValueKind>,
{
    fn from(value: T) -> Self {
        Self { kind: value.into() }
    }
}

#[derive(Debug)]
pub enum ValueKind {
    Object(Object),
    Template(Template),
    String(Quoted),
    Number(f64),
    Boolean(bool),
    Null,
    Undefined,
}

impl From<Object> for ValueKind {
    fn from(value: Object) -> Self {
        Self::Object(value)
    }
}

impl From<&'static str> for ValueKind {
    fn from(value: &'static str) -> Self {
        Self::String(Quoted::new(value))
    }
}

impl From<Template> for ValueKind {
    fn from(value: Template) -> Self {
        Self::Template(value)
    }
}

impl From<String> for ValueKind {
    fn from(value: String) -> Self {
        Self::String(Quoted::new(value))
    }
}

impl From<isize> for ValueKind {
    fn from(value: isize) -> Self {
        Self::Number(value as f64)
    }
}

impl From<bool> for ValueKind {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<f64> for ValueKind {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}
