use itertools::Itertools;

use crate::*;
use std::{
    borrow::Cow,
    fmt::{self, Display, Write},
};

pub(super) const BUILTIN_SCALARS: &[&str] = &["ID", "String", "Int", "Float", "Boolean"];
pub(super) const INDENT: &str = "    ";

/// Lets you take a routine that expects a formatter, and use it on a string.
pub(in crate::render_sdl) fn with_formatter<F>(out: &mut String, action: F) -> fmt::Result
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    struct Helper<T>(T);

    impl<T> Display for Helper<T>
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            (self.0)(f)
        }
    }

    out.write_fmt(format_args!("{}", Helper(action)))
}

#[doc(hidden)]
pub fn display_graphql_string_literal(string: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_char('"')?;
    for c in string.chars() {
        match c {
            '\r' => f.write_str("\\r"),
            '\n' => f.write_str("\\n"),
            '\t' => f.write_str("\\t"),
            '\\' => f.write_str("\\\\"),
            '"' => f.write_str("\\\""),
            c if c.is_control() => write!(f, "\\u{:04}", c as u32),
            c => f.write_char(c),
        }?
    }
    f.write_char('"')
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

        let mut lines = description.lines().skip_while(|line| line.is_empty()).peekable();

        while let Some(line) = lines.next() {
            let line = line.trim();

            if line.is_empty() && lines.peek().map(|next| next.is_empty()).unwrap_or(true) {
                continue;
            }

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
            crate::Value::String(s) => display_graphql_string_literal(&graph[*s], f),
            crate::Value::Int(i) => Display::fmt(i, f),
            crate::Value::Float(val) => Display::fmt(val, f),
            crate::Value::UnboundEnumValue(val) => f.write_str(&graph[*val]),
            crate::Value::EnumValue(val) => f.write_str(&graph[graph[*val].value]),
            crate::Value::Boolean(true) => f.write_str("true"),
            crate::Value::Boolean(false) => f.write_str("false"),
            crate::Value::Object(key_values) => {
                let mut key_values = key_values.iter().peekable();

                f.write_char('{')?;
                while let Some((key, value)) = key_values.next() {
                    f.write_str(graph.str(*key))?;
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

                let mut values = values.as_ref().iter().peekable();
                while let Some(value) = values.next() {
                    ValueDisplay(value, graph).fmt(f)?;
                    if values.peek().is_some() {
                        f.write_str(", ")?;
                    }
                }

                f.write_char(']')
            }
        }
    }
}

struct JsonValueDisplay<'a>(&'a serde_json::Value);

impl fmt::Display for JsonValueDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            serde_json::Value::Null => f.write_str("null"),
            serde_json::Value::String(s) => display_graphql_string_literal(s, f),
            serde_json::Value::Number(num) => Display::fmt(num, f),
            serde_json::Value::Bool(true) => f.write_str("true"),
            serde_json::Value::Bool(false) => f.write_str("false"),
            serde_json::Value::Object(key_values) => {
                let mut key_values = key_values.iter().peekable();

                f.write_char('{')?;
                while let Some((key, value)) = key_values.next() {
                    f.write_str(key)?;
                    f.write_str(": ")?;
                    JsonValueDisplay(value).fmt(f)?;
                    if key_values.peek().is_some() {
                        f.write_str(", ")?;
                    }
                }
                f.write_char('}')
            }
            serde_json::Value::Array(values) => {
                f.write_char('[')?;

                let mut values = values.iter().peekable();
                while let Some(value) = values.next() {
                    JsonValueDisplay(value).fmt(f)?;
                    if values.peek().is_some() {
                        f.write_str(", ")?;
                    }
                }

                f.write_char(']')
            }
        }
    }
}

struct Arguments<'a>(pub &'a [(StringId, Value)], pub &'a FederatedGraph);

impl Display for Arguments<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Arguments(arguments, graph) = self;

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

/// Displays a field set inside quotes
pub(super) struct SelectionSetDisplay<'a>(pub &'a crate::SelectionSet, pub &'a FederatedGraph);

impl Display for SelectionSetDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = format!("{}", BareSelectionSetDisplay(self.0, self.1));
        display_graphql_string_literal(&out, f)
    }
}

pub(super) struct BareSelectionSetDisplay<'a>(pub &'a crate::SelectionSet, pub &'a FederatedGraph);

impl Display for BareSelectionSetDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let BareSelectionSetDisplay(selection_set, graph) = self;
        let mut selections = selection_set.iter().peekable();

        while let Some(selection) = selections.next() {
            match selection {
                Selection::Field(FieldSelection {
                    field_id,
                    arguments,
                    subselection,
                }) => {
                    let name = &graph[graph[*field_id].name];

                    f.write_str(name)?;

                    let arguments = arguments
                        .iter()
                        .map(|(arg, value)| (graph[*arg].name, value.clone()))
                        .collect::<Vec<_>>();

                    Arguments(&arguments, graph).fmt(f)?;

                    if !subselection.is_empty() {
                        f.write_str(" { ")?;
                        BareSelectionSetDisplay(subselection, graph).fmt(f)?;
                        f.write_str(" }")?;
                    }

                    if selections.peek().is_some() {
                        f.write_char(' ')?;
                    }
                }
                Selection::InlineFragment { on, subselection } => {
                    f.write_str("... on ")?;
                    f.write_str(graph.definition_name(*on))?;
                    f.write_str(" { ")?;
                    BareSelectionSetDisplay(subselection, graph).fmt(f)?;
                    f.write_str(" }")?;
                }
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
        display_graphql_string_literal(&out, f)
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

pub(crate) enum AnyValue<'a> {
    Value(Cow<'a, Value>),
    String(Cow<'a, str>),
    Object(Vec<(&'static str, AnyValue<'a>)>),
    List(Vec<AnyValue<'a>>),
    /// Be careful using this - it will not encode enums correctly...
    JsonValue(serde_json::Value),
    FieldSet(SelectionSetDisplay<'a>),
    InputValueDefinitionSet(InputValueDefinitionSetDisplay<'a>),
    DirectiveArguments(&'a [(StringId, Value)]),
}

impl<'a> From<Vec<AnyValue<'a>>> for AnyValue<'a> {
    fn from(v: Vec<AnyValue<'a>>) -> Self {
        Self::List(v)
    }
}

struct DisplayableAnyValue<'a> {
    graph: &'a FederatedGraph,
    value: &'a AnyValue<'a>,
}

impl std::fmt::Display for DisplayableAnyValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            AnyValue::Value(v) => ValueDisplay(v, self.graph).fmt(f),
            AnyValue::JsonValue(v) => JsonValueDisplay(v).fmt(f),
            AnyValue::FieldSet(v) => v.fmt(f),
            AnyValue::InputValueDefinitionSet(v) => v.fmt(f),
            AnyValue::String(s) => display_graphql_string_literal(s, f),
            AnyValue::DirectiveArguments(arguments) => write_arguments(self.graph, f, arguments),
            AnyValue::List(elems) => {
                f.write_char('[')?;

                let mut elems = elems.iter().peekable();

                while let Some(elem) = elems.next() {
                    DisplayableAnyValue {
                        graph: self.graph,
                        value: elem,
                    }
                    .fmt(f)?;

                    if elems.peek().is_some() {
                        f.write_str(", ")?;
                    }
                }

                f.write_char(']')
            }
            AnyValue::Object(items) => {
                f.write_char('{')?;

                let mut items = items.iter().peekable();

                while let Some((key, value)) = items.next() {
                    f.write_str(key)?;
                    f.write_str(": ")?;

                    DisplayableAnyValue {
                        graph: self.graph,
                        value,
                    }
                    .fmt(f)?;

                    if items.peek().is_some() {
                        f.write_str(", ")?;
                    }
                }

                f.write_char('}')
            }
        }
    }
}

impl From<Value> for AnyValue<'_> {
    fn from(value: Value) -> Self {
        AnyValue::Value(Cow::Owned(value))
    }
}

impl<'a> From<SelectionSetDisplay<'a>> for AnyValue<'a> {
    fn from(value: SelectionSetDisplay<'a>) -> Self {
        AnyValue::FieldSet(value)
    }
}

impl<'a> From<InputValueDefinitionSetDisplay<'a>> for AnyValue<'a> {
    fn from(value: InputValueDefinitionSetDisplay<'a>) -> Self {
        AnyValue::InputValueDefinitionSet(value)
    }
}

impl From<serde_json::Value> for AnyValue<'_> {
    fn from(value: serde_json::Value) -> Self {
        AnyValue::JsonValue(value)
    }
}
impl From<String> for AnyValue<'_> {
    fn from(value: String) -> Self {
        AnyValue::String(value.into())
    }
}

impl<'a> From<&'a Vec<(StringId, Value)>> for AnyValue<'a> {
    fn from(value: &'a Vec<(StringId, Value)>) -> Self {
        AnyValue::DirectiveArguments(value.as_slice())
    }
}

impl<'a, 'b> From<&'a str> for AnyValue<'b>
where
    'a: 'b,
{
    fn from(value: &'a str) -> Self {
        AnyValue::String(value.into())
    }
}

pub(crate) struct DirectiveWriter<'a, 'b> {
    f: &'a mut fmt::Formatter<'b>,
    graph: &'a FederatedGraph,
    paren_open: bool,
}

impl<'a, 'b> DirectiveWriter<'a, 'b> {
    pub(crate) fn new(
        directive_name: &str,
        f: &'a mut fmt::Formatter<'b>,
        graph: &'a FederatedGraph,
    ) -> Result<Self, fmt::Error> {
        f.write_str("@")?;
        f.write_str(directive_name)?;

        Ok(DirectiveWriter {
            f,
            graph,
            paren_open: false,
        })
    }

    pub(crate) fn arg<'c>(mut self, name: &str, value: impl Into<AnyValue<'c>>) -> Result<Self, fmt::Error> {
        if !self.paren_open {
            self.f.write_str("(")?;
            self.paren_open = true;
        } else {
            self.f.write_str(", ")?;
        }

        let value = value.into();

        self.f.write_str(name)?;
        self.f.write_str(": ")?;

        write!(
            self.f,
            "{}",
            DisplayableAnyValue {
                graph: self.graph,
                value: &value
            }
        )?;

        Ok(self)
    }
}

impl Drop for DirectiveWriter<'_, '_> {
    fn drop(&mut self) {
        if self.paren_open {
            self.f.write_str(")").ok();
        }
    }
}

pub(super) fn render_field_type(field_type: &Type, graph: &FederatedGraph) -> String {
    let (namespace_id, name_id) = match field_type.definition {
        Definition::Scalar(scalar_id) => {
            let scalar = &graph[scalar_id];

            (scalar.namespace, scalar.name)
        }
        Definition::Object(object_id) => (None, graph.view(object_id).name),
        Definition::Interface(interface_id) => (None, graph.view(interface_id).name),
        Definition::Union(union_id) => (None, graph[union_id].name),
        Definition::Enum(enum_id) => {
            let r#enum = &graph[enum_id];
            (r#enum.namespace, r#enum.name)
        }
        Definition::InputObject(input_object_id) => (None, graph[input_object_id].name),
    };
    let name = &graph[name_id];
    let mut out = String::with_capacity(name.len());

    for _ in 0..field_type.wrapping.list_wrappings().len() {
        out.push('[');
    }

    write!(
        out,
        "{namespace}{separator}{name}",
        namespace = namespace_id.map(|ns| graph[ns].as_str()).unwrap_or(""),
        separator = if namespace_id.is_some() { "__" } else { "" }
    )
    .unwrap();
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

fn write_arguments(graph: &FederatedGraph, f: &mut fmt::Formatter<'_>, arguments: &[(StringId, Value)]) -> fmt::Result {
    write!(
        f,
        "{{{}}}",
        arguments.iter().format_with(", ", |(name, value), f| {
            let value = DisplayableAnyValue {
                graph,
                value: &AnyValue::Value(Cow::Borrowed(value)),
            };
            f(&format_args!(r#"{}: {}"#, graph[*name], value))
        })
    )
}
