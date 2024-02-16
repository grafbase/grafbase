use std::fmt::{Display, Formatter, Result, Write};

use crate::{InputValue, InputValuesContext};

/// Displays the input value with GraphQL syntax.
pub struct GraphqlDisplayableInpuValue<'ctx, Str, Ctx> {
    pub(super) ctx: Ctx,
    pub(super) value: &'ctx InputValue<Str>,
}

impl<'ctx, Str, Ctx> Display for GraphqlDisplayableInpuValue<'ctx, Str, Ctx>
where
    Ctx: InputValuesContext<'ctx, Str>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.value {
            InputValue::Null => f.write_str("null"),
            InputValue::String(s) => write_quoted(self.ctx.get_str(s), f),
            InputValue::Int(n) => write!(f, "{}", n),
            InputValue::BigInt(n) => write!(f, "{}", n),
            InputValue::Float(n) => write!(f, "{}", n),
            InputValue::U64(n) => write!(f, "{}", n),
            InputValue::Boolean(b) => {
                if *b {
                    f.write_str("true")
                } else {
                    f.write_str("false")
                }
            }
            &InputValue::InputObject(fields) => write_object(
                self.ctx.input_values()[fields]
                    .iter()
                    .map(|(input_value_definition_id, value)| {
                        (
                            self.ctx.schema_walker().walk(*input_value_definition_id).name(),
                            GraphqlDisplayableInpuValue { ctx: self.ctx, value },
                        )
                    }),
                f,
            ),
            &InputValue::Map(fields) => write_object(
                self.ctx.input_values()[fields].iter().map(|(key, value)| {
                    (
                        self.ctx.get_str(key),
                        GraphqlDisplayableInpuValue { ctx: self.ctx, value },
                    )
                }),
                f,
            ),
            &InputValue::List(list) => write_list(
                self.ctx.input_values()[list]
                    .iter()
                    .map(|v| GraphqlDisplayableInpuValue {
                        ctx: self.ctx,
                        value: v,
                    }),
                f,
            ),
            InputValue::EnumValue(id) => write!(f, "{}", self.ctx.schema_walker().walk(*id).name()),
            InputValue::UnknownEnumValue(s) => write!(f, "{}", self.ctx.get_str(s)),
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
