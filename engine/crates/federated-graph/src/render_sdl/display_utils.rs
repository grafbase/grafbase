use crate::*;
use std::fmt::{self, Display, Write};

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
            crate::Value::String(s) => write_quoted(f, &graph[*s]),
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
pub(super) struct SelectionSetDisplay<'a>(pub &'a crate::SelectionSet, pub &'a FederatedGraph);

impl Display for SelectionSetDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = format!("{}", BareSelectionSetDisplay(self.0, self.1));
        write_quoted(f, &out)
    }
}

pub(super) struct BareSelectionSetDisplay<'a>(pub &'a crate::SelectionSet, pub &'a FederatedGraph);

impl Display for BareSelectionSetDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let BareSelectionSetDisplay(selection_set, graph) = self;
        let mut selections = selection_set.iter().peekable();

        while let Some(selection) = selections.next() {
            match selection {
                Selection::Field {
                    field,
                    arguments,
                    subselection,
                } => {
                    let name = &graph[graph[*field].name];

                    f.write_str(name)?;

                    let arguments = arguments
                        .iter()
                        .map(|(arg, value)| (graph[*arg].name, value.clone()))
                        .collect::<Vec<_>>();

                    Arguments(&arguments, graph).fmt(f)?;

                    if !subselection.is_empty() {
                        f.write_str(" { ")?;
                        BareSelectionSetDisplay(&subselection, graph).fmt(f)?;
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
                    BareSelectionSetDisplay(&subselection, graph).fmt(f)?;
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

pub(crate) fn write_composed_directive<'a, 'b: 'a>(
    f: &'a mut fmt::Formatter<'b>,
    directive: &Directive,
    graph: &'a FederatedGraph,
) -> fmt::Result {
    match directive {
        Directive::Authenticated => {
            DirectiveWriter::new("authenticated", f, graph)?;
        }
        Directive::Inaccessible => {
            DirectiveWriter::new("inaccessible", f, graph)?;
        }
        Directive::Deprecated { reason } => {
            let directive = DirectiveWriter::new("deprecated", f, graph)?;

            if let Some(reason) = reason {
                directive.arg("reason", Value::String(*reason))?;
            }
        }
        Directive::Policy(policies) => {
            let policies = Value::List(
                policies
                    .iter()
                    .map(|p| Value::List(p.iter().map(|p| Value::String(*p)).collect()))
                    .collect(),
            );

            DirectiveWriter::new("policy", f, graph)?.arg("policies", policies)?;
        }

        Directive::RequiresScopes(scopes) => {
            let scopes = Value::List(
                scopes
                    .iter()
                    .map(|p| Value::List(p.iter().map(|p| Value::String(*p)).collect()))
                    .collect(),
            );

            DirectiveWriter::new("requiresScopes", f, graph)?.arg("scopes", scopes)?;
        }
        Directive::Other { name, arguments } => {
            let mut directive = DirectiveWriter::new(&graph[*name], f, graph)?;

            for (name, value) in arguments {
                directive = directive.arg(&graph[*name], value.clone())?;
            }
        }
    }

    Ok(())
}

pub(crate) enum DisplayableArgument<'a> {
    Value(Value),
    FieldSet(SelectionSetDisplay<'a>),
    InputValueDefinitionSet(InputValueDefinitionSetDisplay<'a>),
    GraphEnumVariantName(GraphEnumVariantName<'a>),
}

impl<'a> DisplayableArgument<'a> {
    pub(crate) fn display(&self, f: &mut fmt::Formatter<'_>, graph: &FederatedGraph) -> fmt::Result {
        match self {
            DisplayableArgument::Value(v) => ValueDisplay(v, graph).fmt(f),
            DisplayableArgument::FieldSet(v) => v.fmt(f),
            DisplayableArgument::InputValueDefinitionSet(v) => v.fmt(f),
            DisplayableArgument::GraphEnumVariantName(inner) => inner.fmt(f),
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
