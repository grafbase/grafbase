use std::fmt::{Display, Formatter, Result, Write};

use crate::{SchemaInputValue, SchemaInputValueWalker};

/// Displays the input value with GraphQL syntax.
impl<'a> Display for SchemaInputValueWalker<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.item {
            SchemaInputValue::Null => f.write_str("null"),
            SchemaInputValue::String(id) => write_quoted(&self.schema[*id], f),
            SchemaInputValue::Int(n) => write!(f, "{}", n),
            SchemaInputValue::BigInt(n) => write!(f, "{}", n),
            SchemaInputValue::Float(n) => write!(f, "{}", n),
            SchemaInputValue::U64(n) => write!(f, "{}", n),
            SchemaInputValue::Boolean(b) => {
                if *b {
                    f.write_str("true")
                } else {
                    f.write_str("false")
                }
            }
            SchemaInputValue::InputObject(ids) => write_object(
                self.schema[*ids].iter().map(|(input_value_definition_id, value)| {
                    let value = self.walk(value);
                    (self.walk(*input_value_definition_id).name(), value)
                }),
                f,
            ),
            SchemaInputValue::Map(ids) => write_object(
                self.schema[*ids].iter().map(|(key, value)| {
                    let value = self.walk(value);
                    (&self.schema[*key], value)
                }),
                f,
            ),
            SchemaInputValue::List(ids) => write_list(self.schema[*ids].iter().map(|value| self.walk(value)), f),
            SchemaInputValue::EnumValue(id) => write!(f, "{}", self.walk(*id).name()),
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
