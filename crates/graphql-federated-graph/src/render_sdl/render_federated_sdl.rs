use itertools::Itertools;

use super::{directive::write_directive, directive_definition::display_directive_definition, display_utils::*};
use crate::{directives::*, federated_graph::*};
use std::fmt::{self, Write};

const JOIN_GRAPH_ENUM_NAME: &str = "join__Graph";

/// Render a GraphQL SDL string for a federated graph. It includes [join spec
/// directives](https://specs.apollo.dev/join/v0.3/) about subgraphs and entities.
pub fn render_federated_sdl(graph: &FederatedGraph) -> Result<String, fmt::Error> {
    let mut sdl = String::new();

    with_formatter(&mut sdl, |f| {
        for definition in &graph.directive_definitions {
            f.write_str("\n")?;
            display_directive_definition(definition, directives_filter, graph, f)?;
        }

        f.write_str("\n")
    })?;

    write_subgraphs_enum(graph, &mut sdl)?;

    for scalar in graph.iter_scalar_definitions() {
        let name = scalar.then(|scalar| scalar.name).as_str();

        if let Some(description) = scalar.description {
            write!(sdl, "{}", Description(&graph[description], ""))?;
        }

        if BUILTIN_SCALARS.contains(&name) {
            continue;
        }

        write!(sdl, "scalar {name}")?;
        write_definition_directives(&scalar.directives, graph, &mut sdl)?;
        sdl.push('\n');
        sdl.push('\n');
    }

    for object in graph.iter_objects() {
        let object_name = &graph[object.name];

        let mut fields = graph[object.fields.clone()]
            .iter()
            .filter(|field| !graph[field.name].starts_with("__"))
            .peekable();

        if fields.peek().is_none() {
            sdl.push_str("\n\n");
            continue;
        }

        if let Some(description) = object.description {
            write!(sdl, "{}", Description(&graph[description], ""))?;
        }

        sdl.push_str("type ");
        sdl.push_str(object_name);

        if !object.implements_interfaces.is_empty() {
            sdl.push_str(" implements ");

            for (idx, interface) in object.implements_interfaces.iter().enumerate() {
                let interface_name = graph.at(*interface).then(|iface| iface.name).as_str();

                sdl.push_str(interface_name);

                if idx < object.implements_interfaces.len() - 1 {
                    sdl.push_str(" & ");
                }
            }
        }

        write_definition_directives(&object.directives, graph, &mut sdl)?;

        if !sdl.ends_with('\n') {
            sdl.push('\n');
        }
        sdl.push_str("{\n");

        for field in fields {
            write_field(&object.directives, field, graph, &mut sdl)?;
        }

        writeln!(sdl, "}}\n")?;
    }

    for interface in graph.iter_interfaces() {
        let interface_name = &graph[interface.name];

        if let Some(description) = interface.description {
            write!(sdl, "{}", Description(&graph[description], ""))?;
        }

        let interface_start = sdl.len();
        write!(sdl, "interface {interface_name}")?;

        if !interface.implements_interfaces.is_empty() {
            sdl.push_str(" implements ");

            for (idx, implemented) in interface.implements_interfaces.iter().enumerate() {
                let implemented_interface = graph.view(*implemented);
                let implemented_interface_name = &graph[implemented_interface.name];
                sdl.push_str(implemented_interface_name);

                if idx < interface.implements_interfaces.len() - 1 {
                    sdl.push_str(" & ");
                }
            }
        }

        let directives_start = sdl.len();
        write_definition_directives(&interface.directives, graph, &mut sdl)?;

        if sdl[interface_start..].len() >= 80 || sdl[directives_start..].len() >= 20 {
            if !sdl.ends_with('\n') {
                sdl.push('\n');
            }
        } else if !sdl.ends_with('\n') && !sdl.ends_with(' ') {
            sdl.push(' ');
        }
        sdl.push_str("{\n");

        for field in &graph[interface.fields.clone()] {
            write_field(&interface.directives, field, graph, &mut sdl)?;
        }

        writeln!(sdl, "}}\n")?;
    }

    for r#enum in graph.iter_enum_definitions() {
        let enum_name = graph.at(r#enum.name).as_str();

        if enum_name == JOIN_GRAPH_ENUM_NAME {
            continue;
        }

        if let Some(description) = r#enum.description {
            write!(sdl, "{}", Description(&graph[description], ""))?;
        }

        write!(sdl, "enum {enum_name}")?;
        write_definition_directives(&r#enum.directives, graph, &mut sdl)?;
        if !sdl.ends_with('\n') {
            sdl.push('\n');
        }
        sdl.push_str("{\n");

        for value in graph.iter_enum_values(r#enum.id()) {
            let value_name = &graph[value.value];

            if let Some(description) = value.description {
                write!(sdl, "{}", Description(&graph[description], INDENT))?;
            }

            write!(sdl, "{INDENT}{value_name}")?;
            with_formatter(&mut sdl, |f| {
                for directive in &value.directives {
                    f.write_str(" ")?;
                    write_directive(f, directive, graph)?;
                }
                Ok(())
            })?;

            sdl.push('\n');
        }

        writeln!(sdl, "}}\n")?;
    }

    for union in &graph.unions {
        let union_name = &graph[r#union.name];

        if let Some(description) = union.description {
            write!(sdl, "{}", Description(&graph[description], ""))?;
        }

        write!(sdl, "union {union_name}")?;

        write_definition_directives(&union.directives, graph, &mut sdl)?;
        if !sdl.ends_with('\n') {
            sdl.push('\n');
        }
        sdl.push_str(" = ");

        let mut members = union.members.iter().peekable();

        while let Some(member) = members.next() {
            sdl.push_str(graph.at(*member).then(|member| member.name).as_str());

            if members.peek().is_some() {
                sdl.push_str(" | ");
            }
        }

        sdl.push_str("\n\n");
    }

    for input_object in &graph.input_objects {
        let name = &graph[input_object.name];

        if let Some(description) = input_object.description {
            write!(sdl, "{}", Description(&graph[description], ""))?;
        }

        write!(sdl, "input {name}")?;

        write_definition_directives(&input_object.directives, graph, &mut sdl)?;
        if !sdl.ends_with('\n') {
            sdl.push('\n');
        }
        sdl.push_str("{\n");

        for field in &graph[input_object.fields] {
            write_input_field(&input_object.directives, field, graph, &mut sdl)?;
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
    if graph.subgraphs.is_empty() {
        return Ok(());
    }

    sdl.push_str("enum join__Graph");

    sdl.push_str(" {\n");

    for subgraph in &graph.subgraphs {
        let name_str = &graph[subgraph.name];
        let loud_name = GraphEnumVariantName(name_str);
        write!(sdl, r#"{INDENT}{loud_name} @join__graph(name: "{name_str}""#)?;
        if let Some(url) = subgraph.url {
            let url = &graph[url];
            write!(sdl, r#", url: "{url}""#)?;
        }
        writeln!(sdl, ")")?;
    }

    sdl.push_str("}\n\n");

    Ok(())
}

fn write_input_field(
    parent_input_object_directives: &[Directive],
    field: &InputValueDefinition,
    graph: &FederatedGraph,
    sdl: &mut String,
) -> fmt::Result {
    let field_name = &graph[field.name];
    let field_type = render_field_type(&field.r#type, graph);

    if let Some(description) = field.description {
        write!(sdl, "{}", Description(&graph[description], INDENT))?;
    }

    write!(sdl, "{INDENT}{field_name}: {field_type}")?;

    if let Some(default) = &field.default {
        write!(sdl, " = {}", ValueDisplay(default, graph))?;
    }

    write_field_directives(parent_input_object_directives, &field.directives, graph, sdl)?;

    sdl.push('\n');
    Ok(())
}

fn write_field(
    parent_entity_directives: &[Directive],
    field: &Field,
    graph: &FederatedGraph,
    sdl: &mut String,
) -> fmt::Result {
    let field_name = &graph[field.name];
    let field_type = render_field_type(&field.r#type, graph);
    let args = render_field_arguments(&graph[field.arguments], graph);

    if let Some(description) = field.description {
        write!(sdl, "{}", Description(&graph[description], INDENT))?;
    }

    write!(sdl, "{INDENT}{field_name}{args}: {field_type}")?;

    write_field_directives(parent_entity_directives, &field.directives, graph, sdl)?;

    sdl.push('\n');
    Ok(())
}

fn write_definition_directives(directives: &[Directive], graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    with_formatter(sdl, |f| {
        for directive in directives {
            f.write_fmt(format_args!("\n{INDENT}"))?;
            write_directive(f, directive, graph)?;
        }

        Ok(())
    })
}

fn write_field_directives(
    parent_type_directives: &[Directive],
    directives: &[Directive],
    graph: &FederatedGraph,
    sdl: &mut String,
) -> fmt::Result {
    // Whether @join__field directives must be present because one of their optional arguments such
    // as requires is present on at least one of them.
    let mut join_field_must_be_present = false;
    let mut join_field_subgraph_ids = Vec::new();
    // Subgraphs which are fully overridden by another one. We don't need to generate a
    // @join__field for those.
    let mut fully_overridden_subgraph_ids = Vec::new();

    for directive in directives {
        if let Directive::JoinField(dir) = directive {
            if let (Some(OverrideSource::Subgraph(id)), None | Some(OverrideLabel::Percent(100))) =
                (dir.r#override.as_ref(), dir.override_label.as_ref())
            {
                fully_overridden_subgraph_ids.push(*id);
            }
            join_field_subgraph_ids.extend(dir.subgraph_id);
            join_field_must_be_present |=
                dir.r#override.is_some() | dir.requires.is_some() | dir.provides.is_some() | dir.r#type.is_some();
        }
    }

    // If there is no use of special arguments of @join_field, we just need to check whether their
    // count matches the number of subgraphs. If so, they're redundant, which is often the case for
    // key fields.
    if !join_field_must_be_present {
        let subgraph_ids = {
            let mut ids = parent_type_directives
                .iter()
                .filter_map(|dir| dir.as_join_type())
                .map(|dir| dir.subgraph_id)
                .collect::<Vec<_>>();
            ids.sort_unstable();
            ids.into_iter().dedup().collect::<Vec<_>>()
        };
        join_field_subgraph_ids.sort_unstable();
        join_field_must_be_present |= subgraph_ids != join_field_subgraph_ids;
    }

    with_formatter(sdl, |f| {
        for directive in directives {
            if let Directive::JoinField(JoinFieldDirective {
                subgraph_id: Some(subgraph_id),
                ..
            }) = &directive
            {
                if !join_field_must_be_present || fully_overridden_subgraph_ids.contains(subgraph_id) {
                    continue;
                }
            }
            f.write_str(" ")?;
            write_directive(f, directive, graph)?;
        }
        Ok(())
    })
}

fn render_field_arguments(args: &[InputValueDefinition], graph: &FederatedGraph) -> String {
    if args.is_empty() {
        String::new()
    } else {
        let mut inner = args
            .iter()
            .map(|arg| {
                let name = &graph[arg.name];
                let r#type = render_field_type(&arg.r#type, graph);
                let directives = &arg.directives;
                let default = arg.default.as_ref();
                let description = arg.description;
                (name, r#type, directives, default, description)
            })
            .peekable();
        let mut out = String::from('(');

        while let Some((name, ty, directives, default, description)) = inner.next() {
            if let Some(description) = description {
                with_formatter(&mut out, |f| {
                    display_graphql_string_literal(&graph[description], f)?;
                    f.write_str(" ")
                })
                .unwrap();
            }

            out.push_str(name);
            out.push_str(": ");
            out.push_str(&ty);

            if let Some(default) = default {
                out.push_str(" = ");
                write!(out, "{}", ValueDisplay(default, graph)).unwrap();
            }

            with_formatter(&mut out, |f| {
                for directive in directives {
                    f.write_str(" ")?;
                    write_directive(f, directive, graph)?;
                }
                Ok(())
            })
            .unwrap();

            if inner.peek().is_some() {
                out.push_str(", ");
            }
        }
        out.push(')');
        out
    }
}

pub(super) struct ListSizeRender<'a> {
    pub list_size: &'a ListSize,
    pub graph: &'a FederatedGraph,
}

impl std::fmt::Display for ListSizeRender<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(" ")?;

        let ListSizeRender {
            graph,
            list_size:
                ListSize {
                    assumed_size,
                    slicing_arguments,
                    sized_fields,
                    require_one_slicing_argument,
                },
        } = self;

        let mut writer = DirectiveWriter::new("listSize", f, graph)?;
        if let Some(size) = assumed_size {
            writer = writer.arg("assumedSize", Value::Int(*size as i64))?;
        }

        if !slicing_arguments.is_empty() {
            let slicing_arguments = slicing_arguments
                .iter()
                .map(|arg| Value::String(graph[*arg].name))
                .collect::<Vec<_>>();

            writer = writer.arg("slicingArguments", Value::List(slicing_arguments.into_boxed_slice()))?;
        }

        if !sized_fields.is_empty() {
            let sized_fields = sized_fields
                .iter()
                .map(|field| Value::String(graph[*field].name))
                .collect::<Vec<_>>();

            writer = writer.arg("sizedFields", Value::List(sized_fields.into_boxed_slice()))?;
        }

        if !require_one_slicing_argument {
            // require_one_slicing_argument defaults to true so we omit it unless its false
            writer.arg(
                "requireOneSlicingArgument",
                Value::Boolean(*require_one_slicing_argument),
            )?;
        }

        Ok(())
    }
}

fn directives_filter(_: &Directive, _: &FederatedGraph) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty() {
        use expect_test::expect;

        let empty = FederatedGraph::default();

        let actual = render_federated_sdl(&empty).expect("valid");
        let expected = expect![[r#"

        "#]];

        expected.assert_eq(&actual);
    }

    #[test]
    fn escape_strings() {
        use expect_test::expect;

        let empty = FederatedGraph::from_sdl(
            r###"
            directive @dummy(test: String!) on FIELD

            type Query {
                field: String @deprecated(reason: "This is a \"deprecated\" reason") @dummy(test: "a \"test\"")
            }
            "###,
        )
        .unwrap();

        let actual = render_federated_sdl(&empty).expect("valid");
        let expected = expect![[r#"

            directive @dummy(test: String!) on FIELD

            type Query
            {
                field: String @deprecated(reason: "This is a \"deprecated\" reason") @dummy(test: "a \"test\"")
            }
        "#]];

        expected.assert_eq(&actual);
    }

    #[test]
    fn multiline_strings() {
        use expect_test::expect;

        let empty = FederatedGraph::from_sdl(
            r###"
            directive @dummy(test: String!) on FIELD

            type Query {
                field: String @deprecated(reason: """This is a "deprecated" reason

                on multiple lines.

                yes, way

                """) @dummy(test: "a \"test\"")
            }
            "###,
        )
        .unwrap();

        let actual = render_federated_sdl(&empty).expect("valid");
        let expected = expect![[r#"

            directive @dummy(test: String!) on FIELD

            type Query
            {
                field: String @deprecated(reason: "This is a \"deprecated\" reason\n\non multiple lines.\n\nyes, way") @dummy(test: "a \"test\"")
            }
        "#]];

        expected.assert_eq(&actual);
    }

    #[test]
    fn regression_empty_keys() {
        // Types that have a @join__type without a key argument should _not_ render with an empty string as a key.
        let schema = r##"
            enum join__Graph {
              a @join__graph(name: "mocksubgraph", url: "https://mock.example.com/todo/graphql")
            }

            interface b @join__type(graph: a) {
              c: String
            }
        "##;

        let parsed = FederatedGraph::from_sdl(schema).unwrap();
        let rendered = render_federated_sdl(&parsed).unwrap();

        let expected = expect_test::expect![[r#"

            enum join__Graph {
                MOCKSUBGRAPH @join__graph(name: "mocksubgraph", url: "https://mock.example.com/todo/graphql")
            }



            interface b
                @join__type(graph: MOCKSUBGRAPH)
            {
                c: String
            }
        "#]];

        expected.assert_eq(&rendered);

        // Check that from_sdl accepts the rendered sdl
        {
            FederatedGraph::from_sdl(&rendered).unwrap();
        }
    }
}
