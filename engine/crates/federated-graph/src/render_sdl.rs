use crate::federated_graph::*;
use std::fmt::{self, Display, Write as _};

const INDENT: &str = "    ";
const BUILTIN_SCALARS: &[&str] = &["ID", "String", "Int", "Float", "Boolean"];

/// Render a GraphQL SDL string for a federated graph. It includes [join spec
/// directives](https://specs.apollo.dev/join/v0.3/) about subgraphs and entities.
pub fn render_sdl(graph: &FederatedGraph) -> Result<String, fmt::Error> {
    let mut sdl = String::new();

    write_subgraphs_enum(graph, &mut sdl)?;

    for scalar in &graph.scalars {
        let name = &graph[scalar.name];

        if BUILTIN_SCALARS.contains(&name.as_str()) {
            continue;
        }

        writeln!(sdl, "scalar {name}\n")?;
    }

    for (idx, object) in graph.objects.iter().enumerate() {
        let object_name = &graph[object.name];

        sdl.push_str("type ");
        sdl.push_str(object_name);

        if !object.implements_interfaces.is_empty() {
            sdl.push_str(" implements ");

            for (idx, interface) in object.implements_interfaces.iter().enumerate() {
                let interface_name = &graph[graph[*interface].name];
                sdl.push_str(interface_name);

                if idx < object.implements_interfaces.len() - 1 {
                    sdl.push_str(" & ");
                }
            }
        }

        if object.resolvable_keys.is_empty() {
            sdl.push_str(" {\n");
        } else {
            sdl.push('\n');
            for resolvable_key in &object.resolvable_keys {
                let selection_set = FieldSetDisplay(&resolvable_key.fields, graph);
                let subgraph_name = GraphEnumVariantName(&graph[graph[resolvable_key.subgraph_id].name]);
                writeln!(
                    sdl,
                    r#"{INDENT}@join__type(graph: {subgraph_name}, key: "{selection_set}")"#
                )?;
            }

            sdl.push_str("{\n");
        }

        for field in graph.object_fields.iter().filter(|field| field.object_id.0 == idx) {
            write_field(field.field_id, graph, &mut sdl)?;
        }

        writeln!(sdl, "}}\n")?;
    }

    for (idx, interface) in graph.interfaces.iter().enumerate() {
        let interface_name = &graph[interface.name];
        write!(sdl, "interface {interface_name}")?;

        if !interface.implements_interfaces.is_empty() {
            sdl.push_str(" implements ");

            for (idx, implemented) in interface.implements_interfaces.iter().enumerate() {
                let implemented_interface_name = &graph[graph[*implemented].name];
                sdl.push_str(implemented_interface_name);

                if idx < interface.implements_interfaces.len() - 1 {
                    sdl.push_str(" & ");
                }
            }
        }

        if interface.resolvable_keys.is_empty() {
            sdl.push_str(" {\n");
        } else {
            sdl.push('\n');
            for resolvable_key in &interface.resolvable_keys {
                let selection_set = FieldSetDisplay(&resolvable_key.fields, graph);
                let subgraph_name = GraphEnumVariantName(&graph[graph[resolvable_key.subgraph_id].name]);
                let is_interface_object = if resolvable_key.is_interface_object {
                    ", isInterfaceObject: true"
                } else {
                    ""
                };
                writeln!(
                    sdl,
                    r#"{INDENT}@join__type(graph: {subgraph_name}, key: "{selection_set}"{is_interface_object})"#
                )?;
            }

            sdl.push_str("{\n");
        }

        for field in graph
            .interface_fields
            .iter()
            .filter(|field| field.interface_id.0 == idx)
        {
            write_field(field.field_id, graph, &mut sdl)?;
        }

        writeln!(sdl, "}}\n")?;
    }

    for r#enum in &graph.enums {
        let enum_name = &graph[r#enum.name];
        writeln!(sdl, "enum {enum_name} {{")?;

        for value in &r#enum.values {
            let value_name = &graph[value.value];
            write!(sdl, "{INDENT}{value_name}")?;

            for directive in &value.composed_directives {
                let directive_name = &graph[directive.name];
                let arguments = DirectiveArguments(&directive.arguments, graph);
                write!(sdl, " @{directive_name}{arguments}")?;
            }

            sdl.push('\n');
        }

        writeln!(sdl, "}}\n")?;
    }

    for union in &graph.unions {
        let union_name = &graph[r#union.name];
        write!(sdl, "union {union_name} = ")?;

        let mut members = union.members.iter().peekable();

        while let Some(member) = members.next() {
            sdl.push_str(&graph[graph[*member].name]);

            if members.peek().is_some() {
                sdl.push_str(" | ");
            }
        }
    }

    for input_object in &graph.input_objects {
        let name = &graph[input_object.name];

        writeln!(sdl, "input {name} {{")?;

        for field in &input_object.fields {
            let field_name = &graph[field.name];
            let field_type = render_field_type(&graph[field.field_type_id], graph);
            writeln!(sdl, "{INDENT}{field_name}: {field_type}")?;
        }

        writeln!(sdl, "}}\n")?;
    }

    // Normalize to a single final newline.
    while let Some('\n') = sdl.chars().next_back() {
        sdl.pop();
    }
    sdl.push('\n');

    Ok(sdl)
}

fn write_subgraphs_enum(graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    sdl.push_str("enum join__Graph {\n");

    for subgraph in &graph.subgraphs {
        let name_str = &graph[subgraph.name];
        let url = &graph[subgraph.url];
        let loud_name = GraphEnumVariantName(name_str);
        writeln!(
            sdl,
            r#"{INDENT}{loud_name} @join__graph(name: "{name_str}", url: "{url}")"#
        )?;
    }

    sdl.push_str("}\n\n");
    Ok(())
}

fn write_field(field_id: FieldId, graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    let field = &graph[field_id];
    let field_name = &graph[field.name];
    let field_type = render_field_type(&graph[field.field_type_id], graph);
    let args = render_field_arguments(&field.arguments, graph);

    write!(sdl, "{INDENT}{field_name}{args}: {field_type}")?;

    if let Some(subgraph) = &field.resolvable_in {
        write_resolvable_in(*subgraph, field, graph, sdl)?;
    }

    write_provides(field, graph, sdl)?;
    write_requires(field, graph, sdl)?;
    write_composed_directives(field, graph, sdl)?;

    sdl.push('\n');
    Ok(())
}

fn write_composed_directives(field: &Field, graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    for directive in &field.composed_directives {
        let directive_name = &graph[directive.name];
        let arguments = DirectiveArguments(&directive.arguments, graph);
        write!(sdl, " @{directive_name}{arguments}")?;
    }

    Ok(())
}

fn write_resolvable_in(subgraph: SubgraphId, field: &Field, graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    let subgraph_name = GraphEnumVariantName(&graph[graph[subgraph].name]);
    let provides = MaybeDisplay(
        field
            .provides
            .iter()
            .find(|provides| provides.subgraph_id == subgraph)
            .map(|fieldset| format!(", provides: \"{}\"", FieldSetDisplay(&fieldset.fields, graph))),
    );
    let requires = MaybeDisplay(
        field
            .requires
            .iter()
            .find(|requires| requires.subgraph_id == subgraph)
            .map(|fieldset| format!(", requires: \"{}\"", FieldSetDisplay(&fieldset.fields, graph))),
    );
    write!(sdl, " @join__field(graph: {subgraph_name}{provides}{requires})")?;

    Ok(())
}

fn write_provides(field: &Field, graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    for provides in field
        .provides
        .iter()
        .filter(|provide| Some(provide.subgraph_id) != field.resolvable_in)
    {
        let subgraph_name = GraphEnumVariantName(&graph[graph[provides.subgraph_id].name]);
        let fields = FieldSetDisplay(&provides.fields, graph);
        write!(sdl, " @join__field(graph: {subgraph_name}, provides: \"{fields}\"")?;
    }

    Ok(())
}

fn write_requires(field: &Field, graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    for requires in field
        .requires
        .iter()
        .filter(|require| Some(require.subgraph_id) != field.resolvable_in)
    {
        let subgraph_name = GraphEnumVariantName(&graph[graph[requires.subgraph_id].name]);
        let fields = FieldSetDisplay(&requires.fields, graph);
        write!(sdl, " @join__field(graph: {subgraph_name}, requires: \"{fields}\"")?;
    }

    Ok(())
}

fn render_field_type(field_type: &FieldType, graph: &FederatedGraph) -> String {
    let maybe_bang = if field_type.inner_is_required { "!" } else { "" };
    let name_id = match field_type.kind {
        Definition::Scalar(scalar_id) => graph[scalar_id].name,
        Definition::Object(object_id) => graph[object_id].name,
        Definition::Interface(interface_id) => graph[interface_id].name,
        Definition::Union(union_id) => graph[union_id].name,
        Definition::Enum(enum_id) => graph[enum_id].name,
        Definition::InputObject(input_object_id) => graph[input_object_id].name,
    };
    let name = &graph[name_id];
    let mut out = format!("{name}{maybe_bang}");

    for wrapper in &field_type.list_wrappers {
        match wrapper {
            ListWrapper::RequiredList => out = format!("[{out}]!"),
            ListWrapper::NullableList => out = format!("[{out}]"),
        }
    }

    out
}

fn render_field_arguments(args: &[FieldArgument], graph: &FederatedGraph) -> String {
    if args.is_empty() {
        String::new()
    } else {
        let mut inner = args
            .iter()
            .map(|arg| (&graph[arg.name], render_field_type(&graph[arg.type_id], graph)))
            .peekable();
        let mut out = String::from('(');

        while let Some((name, ty)) = inner.next() {
            out.push_str(name);
            out.push_str(": ");
            out.push_str(&ty);

            if inner.peek().is_some() {
                out.push_str(", ");
            }
        }
        out.push(')');
        out
    }
}

struct FieldSetDisplay<'a>(&'a FieldSet, &'a FederatedGraph);

impl Display for FieldSetDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FieldSetDisplay(selection_set, graph) = self;
        let mut selection = selection_set.iter().peekable();

        while let Some(field) = selection.next() {
            let name = &graph[graph[field.field].name];

            f.write_str(name)?;

            if !field.subselection.is_empty() {
                f.write_str(" { ")?;
                FieldSetDisplay::fmt(&FieldSetDisplay(&field.subselection, graph), f)?;
                f.write_str(" }")?;
            }

            if selection.peek().is_some() {
                f.write_char(' ')?;
            }
        }

        Ok(())
    }
}

struct GraphEnumVariantName<'a>(&'a str);

impl Display for GraphEnumVariantName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for char in self.0.chars() {
            for upcased in char.to_uppercase() {
                f.write_char(upcased)?;
            }
        }

        Ok(())
    }
}

struct MaybeDisplay<T>(Option<T>);

impl<T: Display> Display for MaybeDisplay<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(inner) = &self.0 {
            Display::fmt(inner, f)?;
        }

        Ok(())
    }
}

struct DirectiveArguments<'a>(&'a [(StringId, Value)], &'a FederatedGraph);

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

struct ValueDisplay<'a>(&'a Value, &'a FederatedGraph);

impl Display for ValueDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ValueDisplay(value, graph) = self;
        match value {
            Value::String(s) => {
                f.write_str("\"")?;
                f.write_str(&graph[*s])?;
                f.write_str("\"")
            }
            Value::Int(i) => Display::fmt(i, f),
            Value::Float(val) | Value::EnumValue(val) => f.write_str(&graph[*val]),
            Value::Boolean(true) => f.write_str("true"),
            Value::Boolean(false) => f.write_str("false"),
            Value::Object(_) => todo!(),
            Value::List(_) => todo!(),
        }
    }
}
