use std::fmt::{Display, Formatter, Result, Write};

use crate::SchemaInputValueRecord;
use readable::Readable;

use super::SchemaInputValue;

/// Displays the input value with GraphQL syntax.
impl<'a> Display for SchemaInputValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let SchemaInputValue { schema, value } = *self;
        match value {
            SchemaInputValueRecord::Null => f.write_str("null"),
            SchemaInputValueRecord::String(id) => write_quoted(&schema[*id], f),
            SchemaInputValueRecord::Int(n) => write!(f, "{}", n),
            SchemaInputValueRecord::BigInt(n) => write!(f, "{}", n),
            SchemaInputValueRecord::Float(n) => write!(f, "{}", n),
            SchemaInputValueRecord::U64(n) => write!(f, "{}", n),
            SchemaInputValueRecord::Boolean(b) => {
                if *b {
                    f.write_str("true")
                } else {
                    f.write_str("false")
                }
            }
            SchemaInputValueRecord::InputObject(ids) => write_object(
                ids.read(schema)
                    .map(|(input_value_definition, value)| (input_value_definition.name(), value)),
                f,
            ),
            SchemaInputValueRecord::Map(ids) => write_object(ids.read(schema), f),
            SchemaInputValueRecord::List(ids) => write_list(ids.read(schema), f),
            SchemaInputValueRecord::EnumValue(id) => write!(f, "{}", id.read(schema).name()),
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

fn write_list<T: Display>(mut iter: impl Iterator<Item = T>, f: &mut Formatter<'_>) -> Result {
    f.write_char('[')?;
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
        write!(f, "{name}:{value}")?;
    }
    for (name, value) in iter {
        f.write_char(',')?;
        write!(f, "{name}:{value}")?;
    }
    f.write_char('}')
}
