use std::fmt::{Display, Formatter, Result, Write};

use crate::{InputValue, SchemaWalker};

pub struct DisplayableInpuValue<'a> {
    pub(super) schema: SchemaWalker<'a, ()>,
    pub(super) value: &'a InputValue,
}

impl<'a> Display for DisplayableInpuValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.value {
            InputValue::Null => f.write_str("null"),
            InputValue::String(s) => write_quoted(s, f),
            InputValue::StringId(id) => write_quoted(&self.schema[*id], f),
            InputValue::Int(n) => write!(f, "{}", n),
            InputValue::BigInt(n) => write!(f, "{}", n),
            InputValue::Float(n) => write!(f, "{}", n),
            InputValue::Boolean(b) => {
                if *b {
                    f.write_str("true")
                } else {
                    f.write_str("false")
                }
            }
            InputValue::Object(fields) => write_object(
                fields.iter().map(|(input_value_definition_id, value)| {
                    (
                        self.schema.walk(*input_value_definition_id).name(),
                        DisplayableInpuValue {
                            schema: self.schema,
                            value,
                        },
                    )
                }),
                f,
            ),
            InputValue::List(list) => write_list(
                list.iter().map(|v| DisplayableInpuValue {
                    schema: self.schema,
                    value: v,
                }),
                f,
            ),
            InputValue::Json(json) => JSONasGraphql(json).fmt(f),
        }
    }
}

fn write_quoted(s: &str, f: &mut Formatter<'_>) -> Result {
    f.write_char('"')?;
    for c in s.chars() {
        match c {
            '\r' => f.write_str("\\r"),
            '\n' => f.write_str("\\n"),
            '\t' => f.write_str("\\t"),
            '"' => f.write_str("\\\""),
            '\\' => f.write_str("\\\\"),
            c if c.is_control() => write!(f, "\\u{:04}", c as u32),
            c => f.write_char(c),
        }?;
    }
    f.write_char('"')
}

fn write_list<T: Display>(list: impl IntoIterator<Item = T>, f: &mut Formatter<'_>) -> Result {
    f.write_char('[')?;
    let mut iter = list.into_iter();
    if let Some(item) = iter.next() {
        item.fmt(f)?;
    }
    for item in iter {
        f.write_char(',')?;
        item.fmt(f)?;
    }
    f.write_char(']')
}

fn write_object<K: Display, V: Display>(object: impl IntoIterator<Item = (K, V)>, f: &mut Formatter<'_>) -> Result {
    f.write_char('{')?;
    let mut iter = object.into_iter();
    if let Some((name, value)) = iter.next() {
        write!(f, "{name}: {value}")?;
    }
    for (name, value) in iter {
        f.write_char(',')?;
        write!(f, "{name}: {value}")?;
    }
    f.write_char('}')
}

struct JSONasGraphql<'a>(&'a serde_json::Value);

impl<'a> Display for JSONasGraphql<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0 {
            serde_json::Value::Null => f.write_str("null"),
            serde_json::Value::Bool(b) => {
                if *b {
                    f.write_str("true")
                } else {
                    f.write_str("false")
                }
            }
            serde_json::Value::Number(n) => write!(f, "{}", n),
            serde_json::Value::String(s) => f.write_str(s),
            serde_json::Value::Array(list) => write_list(list.iter().map(JSONasGraphql), f),
            serde_json::Value::Object(fields) => {
                write_object(fields.iter().map(|(name, value)| (name, JSONasGraphql(value))), f)
            }
        }
    }
}
