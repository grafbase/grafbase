use crate::*;
use std::fmt::{self, Display, Write};

pub(super) const INDENT: &str = "    ";

// Copy-pasted from async-graphql-value
pub(super) fn write_quoted(sdl: &mut impl Write, s: &str) -> fmt::Result {
    sdl.write_char('"')?;
    for c in s.chars() {
        match c {
            c @ ('\r' | '\n' | '\t' | '"' | '\\') => {
                sdl.write_char('\\')?;
                sdl.write_char(c)
            }
            c if c.is_control() => write!(sdl, "\\u{:04}", c as u32),
            c => sdl.write_char(c),
        }?
    }
    sdl.write_char('"')
}

pub(super) fn write_block(
    f: &mut fmt::Formatter<'_>,
    inner: impl FnOnce(&mut fmt::Formatter<'_>) -> fmt::Result,
) -> fmt::Result {
    write_delimited(f, "{\n", '}', inner)
}

fn write_delimited(
    f: &mut fmt::Formatter<'_>,
    start: &str,
    end: char,
    inner: impl FnOnce(&mut fmt::Formatter<'_>) -> fmt::Result,
) -> fmt::Result {
    f.write_str(start)?;
    inner(f)?;
    f.write_char(end)
}

pub(crate) struct Description<'a>(pub &'a str, pub &'a str);

impl fmt::Display for Description<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Description(description, indentation) = self;

        writeln!(f, r#"{indentation}""""#)?;

        for line in description.lines() {
            writeln!(f, r#"{indentation}{line}"#)?;
        }

        writeln!(f, r#"{indentation}""""#)
    }
}

pub(super) struct ValueDisplay<'a>(pub &'a crate::Value, pub &'a FederatedGraphV3);

impl fmt::Display for ValueDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ValueDisplay(value, graph) = self;
        match value {
            crate::Value::Null => f.write_str("null"),
            crate::Value::String(s) => write_quoted(f, &graph[*s]),
            crate::Value::Int(i) => Display::fmt(i, f),
            crate::Value::Float(val) => Display::fmt(val, f),
            crate::Value::EnumValue(val) => f.write_str(&graph[*val]),
            crate::Value::Boolean(true) => f.write_str("true"),
            crate::Value::Boolean(false) => f.write_str("false"),
            crate::Value::Object(_) => todo!(),
            crate::Value::List(_) => todo!(),
        }
    }
}

pub(super) struct DirectiveArguments<'a>(pub &'a [(StringId, Value)], pub &'a FederatedGraphV3);

impl Display for DirectiveArguments<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let DirectiveArguments(arguments, graph) = self;

        if arguments.is_empty() {
            return Ok(());
        }

        f.write_str("(")?;

        let mut arguments = arguments.iter().peekable();

        while let Some((name, value)) = arguments.next() {
            let name = &graph[*name];
            let value = ValueDisplay(value, graph);
            write!(f, "{name}: {value}")?;

            if arguments.peek().is_some() {
                f.write_str(", ")?;
            }
        }

        f.write_str(")")
    }
}

pub(super) struct MaybeDisplay<T>(pub Option<T>);

impl<T: Display> Display for MaybeDisplay<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(inner) = &self.0 {
            Display::fmt(inner, f)?;
        }

        Ok(())
    }
}

/// Displays a field set inside quotes
pub(super) struct FieldSetDisplay<'a>(pub &'a crate::FieldSet, pub &'a FederatedGraphV3);

impl Display for FieldSetDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = format!("{}", BareFieldSetDisplay(self.0, self.1));
        write_quoted(f, &out)
    }
}

pub(super) struct BareFieldSetDisplay<'a>(pub &'a crate::FieldSet, pub &'a FederatedGraphV3);

impl Display for BareFieldSetDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let BareFieldSetDisplay(selection_set, graph) = self;
        let mut selection = selection_set.iter().peekable();

        while let Some(field) = selection.next() {
            let name = &graph[graph[field.field].name];

            f.write_str(name)?;

            let arguments = field
                .arguments
                .iter()
                .map(|(arg, value)| (graph[*arg].name, value.clone()))
                .collect::<Vec<_>>();

            DirectiveArguments(&arguments, graph).fmt(f)?;

            if !field.subselection.is_empty() {
                f.write_str(" { ")?;
                BareFieldSetDisplay(&field.subselection, graph).fmt(f)?;
                f.write_str(" }")?;
            }

            if selection.peek().is_some() {
                f.write_char(' ')?;
            }
        }

        Ok(())
    }
}

pub(super) fn write_enum_variant(
    f: &mut fmt::Formatter<'_>,
    enum_variant: &EnumValue,
    graph: &FederatedGraphV3,
) -> fmt::Result {
    f.write_str(INDENT)?;
    f.write_str(&graph[enum_variant.value])?;
    f.write_char('\n')
}
