use crate::*;
use std::{
    fmt::{self, Display, Write},
    iter,
};

pub(super) const BUILTIN_SCALARS: &[&str] = &["ID", "String", "Int", "Float", "Boolean"];
pub(super) const INDENT: &str = "    ";

pub(super) fn write_quoted(sdl: &mut impl Write, s: &str) -> fmt::Result {
    sdl.write_char('"')?;
    for c in s.chars() {
        match c {
            '\r' => sdl.write_str("\\r"),
            '\n' => sdl.write_str("\\n"),
            '\t' => sdl.write_str("\\t"),
            '\\' => sdl.write_str("\\\\"),
            '"' => sdl.write_str("\\\""),
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

pub(super) struct ValueDisplay<'a>(pub &'a crate::Value, pub &'a FederatedGraph);

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
            crate::Value::Object(key_values) => {
                let mut key_values = key_values.iter().peekable();

                f.write_char('{')?;
                while let Some((key, value)) = key_values.next() {
                    write_quoted(f, &graph[*key])?;
                    f.write_str(": ")?;
                    ValueDisplay(value, graph).fmt(f)?;
                    if key_values.peek().is_some() {
                        f.write_str(", ")?;
                    }
                }
                f.write_char('}')
            }
            crate::Value::List(values) => {
                f.write_char('[')?;

                for value in values.as_ref() {
                    ValueDisplay(value, graph).fmt(f)?;
                    f.write_str(", ")?;
                }

                f.write_char(']')
            }
        }
    }
}

pub(super) struct DirectiveArguments<'a>(pub &'a [(StringId, Value)], pub &'a FederatedGraph);

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
pub(super) struct FieldSetDisplay<'a>(pub &'a crate::FieldSet, pub &'a FederatedGraph);

impl Display for FieldSetDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = format!("{}", BareFieldSetDisplay(self.0, self.1));
        write_quoted(f, &out)
    }
}

pub(super) struct BareFieldSetDisplay<'a>(pub &'a crate::FieldSet, pub &'a FederatedGraph);

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

/// Displays a input value definition set inside quotes
pub(super) struct InputValueDefinitionSetDisplay<'a>(pub &'a crate::InputValueDefinitionSet, pub &'a FederatedGraph);

impl Display for InputValueDefinitionSetDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = format!("{}", BareInputValueDefinitionSetDisplay(self.0, self.1));
        write_quoted(f, &out)
    }
}

pub(super) struct BareInputValueDefinitionSetDisplay<'a>(
    pub &'a crate::InputValueDefinitionSet,
    pub &'a FederatedGraph,
);

impl Display for BareInputValueDefinitionSetDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let BareInputValueDefinitionSetDisplay(selection_set, graph) = self;
        let mut selection = selection_set.iter().peekable();

        while let Some(field) = selection.next() {
            let name = &graph[graph[field.input_value_definition].name];

            f.write_str(name)?;

            if !field.subselection.is_empty() {
                f.write_str(" { ")?;
                BareInputValueDefinitionSetDisplay(&field.subselection, graph).fmt(f)?;
                f.write_str(" }")?;
            }

            if selection.peek().is_some() {
                f.write_char(' ')?;
            }
        }

        Ok(())
    }
}

pub(super) fn write_description(
    f: &mut fmt::Formatter<'_>,
    description: Option<StringId>,
    indent: &str,
    graph: &FederatedGraph,
) -> fmt::Result {
    let Some(description) = description else { return Ok(()) };
    Display::fmt(&Description(&graph[description], indent), f)
}

pub(crate) struct DirectiveDisplay<'a>(pub &'a Directive, pub &'a FederatedGraph);

impl Display for DirectiveDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let DirectiveDisplay(directive, graph) = self;
        write_composed_directive(f, directive, graph)
    }
}

pub(crate) fn write_composed_directive(
    f: &mut fmt::Formatter<'_>,
    directive: &Directive,
    graph: &FederatedGraph,
) -> fmt::Result {
    match directive {
        Directive::Authenticated => write_directive(f, "authenticated", iter::empty::<(&str, Value)>(), graph),
        Directive::Inaccessible => write_directive(f, "inaccessible", iter::empty::<(&str, Value)>(), graph),
        Directive::Deprecated { reason } => write_directive(
            f,
            "deprecated",
            reason.iter().map(|reason| ("reason", Value::String(*reason))),
            graph,
        ),
        Directive::Policy(policies) => write_directive(
            f,
            "policy",
            std::iter::once((
                "policies",
                Value::List(
                    policies
                        .iter()
                        .map(|p| Value::List(p.iter().map(|p| Value::String(*p)).collect()))
                        .collect(),
                ),
            )),
            graph,
        ),
        Directive::RequiresScopes(scopes) => write_directive(
            f,
            "requiresScopes",
            std::iter::once((
                "scopes",
                Value::List(
                    scopes
                        .iter()
                        .map(|p| Value::List(p.iter().map(|p| Value::String(*p)).collect()))
                        .collect(),
                ),
            )),
            graph,
        ),
        Directive::Other { name, arguments } => write_directive(
            f,
            &graph[*name],
            arguments
                .iter()
                .map(|(name, value)| (graph[*name].as_str(), value.clone())),
            graph,
        ),
    }
}

enum DisplayableArgument<'a> {
    Value(Value),
    FieldSet(FieldSetDisplay<'a>),
    InputValueDefinitionSet(InputValueDefinitionSetDisplay<'a>),
}

impl From<Value> for DisplayableArgument<'_> {
    fn from(value: Value) -> Self {
        DisplayableArgument::Value(value)
    }
}

impl<'a> From<FieldSetDisplay<'a>> for DisplayableArgument<'a> {
    fn from(value: FieldSetDisplay<'a>) -> Self {
        DisplayableArgument::FieldSet(value)
    }
}

impl<'a> From<InputValueDefinitionSetDisplay<'a>> for DisplayableArgument<'a> {
    fn from(value: InputValueDefinitionSetDisplay<'a>) -> Self {
        DisplayableArgument::InputValueDefinitionSet(value)
    }
}

pub(super) struct AuthorizedDirectiveDisplay<'a>(pub &'a AuthorizedDirective, pub &'a FederatedGraph);

impl Display for AuthorizedDirectiveDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let AuthorizedDirectiveDisplay(directive, graph) = self;

        let fields = directive
            .fields
            .as_ref()
            .map(|fields| ("fields", DisplayableArgument::from(FieldSetDisplay(fields, graph))));

        let node = directive
            .node
            .as_ref()
            .map(|fields| ("node", DisplayableArgument::from(FieldSetDisplay(fields, graph))));

        let arguments = directive.arguments.as_ref().map(|arguments| {
            (
                "arguments",
                DisplayableArgument::from(InputValueDefinitionSetDisplay(arguments, graph)),
            )
        });

        let metadata = directive
            .metadata
            .as_ref()
            .map(|metadata| ("metadata", DisplayableArgument::Value(metadata.clone())));

        let arguments = [fields, node, arguments, metadata];

        write_directive(f, "authorized", arguments.into_iter().flatten(), graph)
    }
}

fn write_directive<'a, A>(
    f: &mut fmt::Formatter<'_>,
    directive_name: &str,
    arguments: impl Iterator<Item = (&'a str, A)>,
    graph: &FederatedGraph,
) -> fmt::Result
where
    A: Into<DisplayableArgument<'a>>,
{
    f.write_str(" @")?;
    f.write_str(directive_name)?;
    write_directive_arguments(f, arguments.map(|(name, value)| (name, value.into())), graph)
}

fn write_directive_arguments<'a>(
    f: &mut fmt::Formatter<'_>,
    arguments: impl Iterator<Item = (&'a str, DisplayableArgument<'a>)>,
    graph: &FederatedGraph,
) -> fmt::Result {
    let mut arguments = arguments.peekable();

    if arguments.peek().is_none() {
        return Ok(());
    }

    f.write_str("(")?;

    while let Some((name, value)) = arguments.next() {
        f.write_str(name)?;
        f.write_str(": ")?;

        match value {
            DisplayableArgument::Value(v) => {
                ValueDisplay(&v, graph).fmt(f)?;
            }
            DisplayableArgument::FieldSet(v) => {
                v.fmt(f)?;
            }
            DisplayableArgument::InputValueDefinitionSet(v) => {
                v.fmt(f)?;
            }
        }

        if arguments.peek().is_some() {
            f.write_str(", ")?;
        }
    }

    f.write_str(")")
}

pub(super) fn render_field_type(field_type: &Type, graph: &FederatedGraph) -> String {
    let name_id = match field_type.definition {
        Definition::Scalar(scalar_id) => graph[scalar_id].name,
        Definition::Object(object_id) => graph[object_id].name,
        Definition::Interface(interface_id) => graph[interface_id].name,
        Definition::Union(union_id) => graph[union_id].name,
        Definition::Enum(enum_id) => graph[enum_id].name,
        Definition::InputObject(input_object_id) => graph[input_object_id].name,
    };
    let name = &graph[name_id];
    let mut out = String::with_capacity(name.len());

    for _ in 0..field_type.wrapping.list_wrappings().len() {
        out.push('[');
    }

    write!(out, "{name}").unwrap();
    if field_type.wrapping.inner_is_required() {
        out.push('!');
    }

    for wrapping in field_type.wrapping.list_wrappings() {
        out.push(']');
        if wrapping == wrapping::ListWrapping::RequiredList {
            out.push('!');
        }
    }

    out
}
