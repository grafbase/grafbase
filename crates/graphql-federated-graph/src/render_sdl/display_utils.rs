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

pub(crate) enum DisplayableArgument<'a> {
    Value(Value),
    String(Cow<'a, str>),
    /// Be careful using this - it will not encode enums correctly...
    JsonValue(serde_json::Value),
    FieldSet(SelectionSetDisplay<'a>),
    InputValueDefinitionSet(InputValueDefinitionSetDisplay<'a>),
    GraphEnumVariantName(GraphEnumVariantName<'a>),
}

impl DisplayableArgument<'_> {
    pub(crate) fn display(&self, f: &mut fmt::Formatter<'_>, graph: &FederatedGraph) -> fmt::Result {
        match self {
            DisplayableArgument::Value(v) => ValueDisplay(v, graph).fmt(f),
            DisplayableArgument::JsonValue(v) => JsonValueDisplay(v).fmt(f),
            DisplayableArgument::FieldSet(v) => v.fmt(f),
            DisplayableArgument::InputValueDefinitionSet(v) => v.fmt(f),
            DisplayableArgument::GraphEnumVariantName(inner) => inner.fmt(f),
            DisplayableArgument::String(s) => display_graphql_string_literal(s, f),
        }
    }
}

impl<'a> From<GraphEnumVariantName<'a>> for DisplayableArgument<'a> {
    fn from(value: GraphEnumVariantName<'a>) -> Self {
        DisplayableArgument::GraphEnumVariantName(value)
    }
}

impl From<Value> for DisplayableArgument<'_> {
    fn from(value: Value) -> Self {
        DisplayableArgument::Value(value)
    }
}

impl<'a> From<SelectionSetDisplay<'a>> for DisplayableArgument<'a> {
    fn from(value: SelectionSetDisplay<'a>) -> Self {
        DisplayableArgument::FieldSet(value)
    }
}

impl<'a> From<InputValueDefinitionSetDisplay<'a>> for DisplayableArgument<'a> {
    fn from(value: InputValueDefinitionSetDisplay<'a>) -> Self {
        DisplayableArgument::InputValueDefinitionSet(value)
    }
}

impl From<serde_json::Value> for DisplayableArgument<'_> {
    fn from(value: serde_json::Value) -> Self {
        DisplayableArgument::JsonValue(value)
    }
}
impl From<String> for DisplayableArgument<'_> {
    fn from(value: String) -> Self {
        DisplayableArgument::String(value.into())
    }
}

impl<'a, 'b> From<&'a str> for DisplayableArgument<'b>
where
    'a: 'b,
{
    fn from(value: &'a str) -> Self {
        DisplayableArgument::String(value.into())
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

    pub(crate) fn arg<'c>(mut self, name: &str, value: impl Into<DisplayableArgument<'c>>) -> Result<Self, fmt::Error> {
        if !self.paren_open {
            self.f.write_str("(")?;
            self.paren_open = true;
        } else {
            self.f.write_str(", ")?;
        }

        let value = value.into();

        self.f.write_str(name)?;
        self.f.write_str(": ")?;
        value.display(self.f, self.graph)?;

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

pub(in crate::render_sdl) struct GraphEnumVariantName<'a>(pub &'a str);

impl Display for GraphEnumVariantName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for char in self.0.chars() {
            match char {
                '-' | '_' | ' ' => f.write_char('_')?,
                other => {
                    for char in other.to_uppercase() {
                        f.write_char(char)?;
                    }
                }
            }
        }

        Ok(())
    }
}
