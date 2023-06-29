mod object;

pub use object::Object;
use std::fmt;

use crate::typescript_ast::common::{Quoted, Template};

pub struct Value<'a> {
    kind: ValueKind<'a>,
}

impl<'a> fmt::Display for Value<'a> {
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

impl<'a, T> From<T> for Value<'a>
where
    T: Into<ValueKind<'a>>,
{
    fn from(value: T) -> Self {
        Self { kind: value.into() }
    }
}

#[allow(dead_code)]
pub enum ValueKind<'a> {
    Object(Object<'a>),
    Template(Template<'a>),
    String(Quoted<'a>),
    Number(f64),
    Boolean(bool),
    Null,
    Undefined,
}

impl<'a> From<Object<'a>> for ValueKind<'a> {
    fn from(value: Object<'a>) -> Self {
        Self::Object(value)
    }
}

impl<'a> From<&'a str> for ValueKind<'a> {
    fn from(value: &'a str) -> Self {
        Self::String(Quoted::new(value))
    }
}

impl<'a> From<Template<'a>> for ValueKind<'a> {
    fn from(value: Template<'a>) -> Self {
        Self::Template(value)
    }
}

impl From<String> for ValueKind<'static> {
    fn from(value: String) -> Self {
        Self::String(Quoted::new(value))
    }
}

impl From<isize> for ValueKind<'static> {
    fn from(value: isize) -> Self {
        Self::Number(value as f64)
    }
}

impl From<bool> for ValueKind<'static> {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<f64> for ValueKind<'static> {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

#[cfg(test)]
mod tests {
    use super::Value;
    use crate::test_helpers::{expect, expect_ts};

    #[test]
    fn string_value() {
        let value = Value::from("foo");

        let expected = expect![[r#"
            'foo'
        "#]];

        expect_ts(&value, &expected);
    }

    #[test]
    fn float_value() {
        let value = Value::from(1.23f64);

        let expected = expect![[r#"
            1.23
        "#]];

        expect_ts(&value, &expected);
    }

    #[test]
    fn rounded_float_value() {
        let value = Value::from(3.0f64);

        let expected = expect![[r#"
            3
        "#]];

        expect_ts(&value, &expected);
    }

    #[test]
    fn nan_float_value() {
        let value = Value::from(f64::NAN);

        let expected = expect![[r#"
            NaN
        "#]];

        expect_ts(&value, &expected);
    }

    #[test]
    fn infinite_float_value() {
        let value = Value::from(f64::INFINITY);

        let expected = expect![[r#"
            Infinity
        "#]];

        expect_ts(&value, &expected);
    }

    #[test]
    fn neg_infinite_float_value() {
        let value = Value::from(f64::NEG_INFINITY);

        let expected = expect![[r#"
            ;-Infinity
        "#]];

        expect_ts(&value, &expected);
    }
}
