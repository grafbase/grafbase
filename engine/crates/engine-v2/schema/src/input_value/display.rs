use std::fmt::{Display, Formatter, Result, Write};

use crate::{RawInputValue, RawInputValueWalker, RawInputValuesContext};

/// Displays the input value with GraphQL syntax.
impl<'ctx, Ctx> Display for RawInputValueWalker<'ctx, Ctx>
where
    Ctx: RawInputValuesContext<'ctx>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.value {
            RawInputValue::Null | RawInputValue::Undefined => f.write_str("null"),
            RawInputValue::String(s) => write_quoted(self.ctx.get_str(s), f),
            RawInputValue::Int(n) => write!(f, "{}", n),
            RawInputValue::BigInt(n) => write!(f, "{}", n),
            RawInputValue::Float(n) => write!(f, "{}", n),
            RawInputValue::U64(n) => write!(f, "{}", n),
            RawInputValue::Boolean(b) => {
                if *b {
                    f.write_str("true")
                } else {
                    f.write_str("false")
                }
            }
            RawInputValue::InputObject(ids) => write_object(
                self.ctx.input_values()[*ids]
                    .iter()
                    .filter_map(|(input_value_definition_id, value)| {
                        let value = self.walk(value);
                        if value.is_undefined() {
                            None
                        } else {
                            Some((self.ctx.schema_walker().walk(*input_value_definition_id).name(), value))
                        }
                    }),
                f,
            ),
            RawInputValue::Map(ids) => write_object(
                self.ctx.input_values()[*ids].iter().filter_map(|(key, value)| {
                    let value = self.walk(value);
                    if value.is_undefined() {
                        None
                    } else {
                        Some((self.ctx.get_str(key), value))
                    }
                }),
                f,
            ),
            RawInputValue::List(ids) => write_list(ids.map(|id| self.ctx.walk(id)), f),
            RawInputValue::EnumValue(id) => write!(f, "{}", self.ctx.schema_walker().walk(*id).name()),
            RawInputValue::UnknownEnumValue(s) => write!(f, "{}", self.ctx.get_str(s)),
            RawInputValue::Ref(id) => self.ctx.input_value_ref_display(*id).fmt(f),
            RawInputValue::SchemaRef(id) => self.ctx.schema_walk(*id).fmt(f),
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
