use super::display_utils::*;
use crate::federated_graph::*;
use std::fmt::{self, Write};

/// Render a GraphQL SDL string for a federated graph. It includes [join spec
/// directives](https://specs.apollo.dev/join/v0.3/) about subgraphs and entities.
pub fn render_federated_sdl(graph: &FederatedGraph) -> Result<String, fmt::Error> {
    let mut sdl = String::new();

    write_prelude(&mut sdl)?;

    write_subgraphs_enum(graph, &mut sdl)?;

    for scalar in &graph.scalars {
        let name = &graph[scalar.name];

        if let Some(description) = scalar.description {
            write!(sdl, "{}", Description(&graph[description], ""))?;
        }

        if BUILTIN_SCALARS.contains(&name.as_str()) {
            continue;
        }

        write!(sdl, "scalar {name}")?;
        write_composed_directives(scalar.composed_directives, graph, &mut sdl)?;
        sdl.push('\n');
        sdl.push('\n');
    }

    for object in graph.iter_objects() {
        let definition = graph.at(object.type_definition_id);
        let object_name = definition.then(|def| def.name).as_str();

        let mut fields = graph[object.fields.clone()]
            .iter()
            .enumerate()
            .filter(|(_idx, field)| !graph[field.name].starts_with("__"))
            .peekable();

        if fields.peek().is_none() {
            sdl.push_str("\n\n");
            continue;
        }

        if let Some(description) = definition.description {
            write!(sdl, "{}", Description(&graph[description], ""))?;
        }

        sdl.push_str("type ");
        sdl.push_str(object_name);

        if !object.implements_interfaces.is_empty() {
            sdl.push_str(" implements ");

            for (idx, interface) in object.implements_interfaces.iter().enumerate() {
                let interface_name = graph
                    .at(*interface)
                    .then(|interface| interface.type_definition_id)
                    .then(|def| def.name)
                    .as_str();

                sdl.push_str(interface_name);

                if idx < object.implements_interfaces.len() - 1 {
                    sdl.push_str(" & ");
                }
            }
        }

        with_formatter(&mut sdl, |f| {
            render_composed_directives(definition.directives, f, graph)?;

            for authorized_directive in graph.object_authorized_directives(object.id()) {
                render_authorized_directive(authorized_directive, f, graph)?;
            }

            if !object.join_implements.is_empty() {
                for (subgraph_id, interface_id) in &object.join_implements {
                    f.write_str("\n")?;
                    render_join_implement(*subgraph_id, *interface_id, f, graph)?;
                }

                if object.keys.is_empty() {
                    f.write_str("\n")?;
                }
            }

            if !object.keys.is_empty() {
                f.write_str("\n")?;
                for key in &object.keys {
                    render_join_field(key, f, graph)?;
                }
            } else {
                f.write_str(" ")?;
            }

            Ok(())
        })?;

        sdl.push_str("{\n");

        for (idx, field) in fields {
            let field_id = FieldId(object.fields.start.0 + idx);
            write_field(field_id, field, graph, &mut sdl)?;
        }

        writeln!(sdl, "}}\n")?;
    }

    for interface in graph.iter_interfaces() {
        let definition = graph.at(interface.type_definition_id);
        let interface_name = definition.then(|def| def.name).as_str();

        if let Some(description) = definition.description {
            write!(sdl, "{}", Description(&graph[description], ""))?;
        }

        write!(sdl, "interface {interface_name}")?;

        if !interface.implements_interfaces.is_empty() {
            sdl.push_str(" implements ");

            for (idx, implemented) in interface.implements_interfaces.iter().enumerate() {
                let implemented_interface = graph.view(*implemented);
                let implemented_interface_name = &graph[graph.view(implemented_interface.type_definition_id).name];
                sdl.push_str(implemented_interface_name);

                if idx < interface.implements_interfaces.len() - 1 {
                    sdl.push_str(" & ");
                }
            }
        }

        with_formatter(&mut sdl, |f| {
            for authorized_directive in graph.interface_authorized_directives(interface.id()) {
                render_authorized_directive(authorized_directive, f, graph)?;
            }

            render_composed_directives(definition.directives, f, graph)?;

            if !interface.join_implements.is_empty() {
                for (subgraph_id, interface_id) in &interface.join_implements {
                    f.write_str("\n")?;
                    render_join_implement(*subgraph_id, *interface_id, f, graph)?;
                }

                if interface.keys.is_empty() {
                    f.write_str("\n")?;
                }
            }

            if interface.keys.is_empty() {
                f.write_str(" {\n")
            } else {
                f.write_str("\n")?;
                for key in &interface.keys {
                    render_join_field(key, f, graph)?;
                }

                f.write_str("{\n")
            }
        })?;

        for (idx, field) in graph[interface.fields.clone()].iter().enumerate() {
            let field_id = FieldId(interface.fields.start.0 + idx);
            write_field(field_id, field, graph, &mut sdl)?;
        }

        writeln!(sdl, "}}\n")?;
    }

    for r#enum in &graph.enums {
        let enum_name = &graph[r#enum.name];

        if let Some(description) = r#enum.description {
            write!(sdl, "{}", Description(&graph[description], ""))?;
        }

        write!(sdl, "enum {enum_name}")?;
        write_composed_directives(r#enum.composed_directives, graph, &mut sdl)?;
        sdl.push_str(" {\n");

        for value in &graph[r#enum.values] {
            let value_name = &graph[value.value];

            if let Some(description) = value.description {
                write!(sdl, "{}", Description(&graph[description], INDENT))?;
            }

            write!(sdl, "{INDENT}{value_name}")?;
            write_composed_directives(value.composed_directives, graph, &mut sdl)?;

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
        write_composed_directives(union.composed_directives, graph, &mut sdl)?;

        if !union.join_members.is_empty() {
            with_formatter(&mut sdl, |f| {
                for (subgraph_id, object_id) in &union.join_members {
                    f.write_str("\n")?;
                    render_join_member(*subgraph_id, *object_id, graph, f)?;
                }

                f.write_str("\n")?;

                Ok(())
            })?
        }

        sdl.push_str(" = ");

        let mut members = union.members.iter().peekable();

        while let Some(member) = members.next() {
            sdl.push_str(
                graph
                    .at(*member)
                    .then(|member| member.type_definition_id)
                    .then(|def| def.name)
                    .as_str(),
            );

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

        write_composed_directives(input_object.composed_directives, graph, &mut sdl)?;

        sdl.push_str(" {\n");

        for field in &graph[input_object.fields] {
            write_input_field(field, graph, &mut sdl)?;
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

fn render_join_member(
    subgraph_id: SubgraphId,
    object_id: ObjectId,
    graph: &FederatedGraph,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let subgraph_name = GraphEnumVariantName(&graph[graph[subgraph_id].name]);

    f.write_str(INDENT)?;

    DirectiveWriter::new("join__unionMember", f, graph)?
        .arg("graph", subgraph_name)?
        .arg(
            "member",
            Value::String(graph.at(object_id).then(|object| object.type_definition_id).name),
        )?;

    Ok(())
}

fn write_prelude(sdl: &mut String) -> fmt::Result {
    sdl.push_str(indoc::indoc! {r#"
        directive @core(feature: String!) repeatable on SCHEMA

        directive @join__owner(graph: join__Graph!) on OBJECT

        directive @join__type(
            graph: join__Graph!
            key: String!
            resolvable: Boolean = true
        ) repeatable on OBJECT | INTERFACE

        directive @join__field(
            graph: join__Graph
            requires: String
            provides: String
        ) on FIELD_DEFINITION

        directive @join__graph(name: String!, url: String!) on ENUM_VALUE

        directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE

        directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION
    "#});

    sdl.push('\n');
    Ok(())
}

fn write_subgraphs_enum(graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    if graph.subgraphs.is_empty() {
        return Ok(());
    }

    sdl.push_str("enum join__Graph");

    sdl.push_str(" {\n");

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

fn write_input_field(field: &InputValueDefinition, graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    let field_name = &graph[field.name];
    let field_type = render_field_type(&field.r#type, graph);

    if let Some(description) = field.description {
        write!(sdl, "{}", Description(&graph[description], INDENT))?;
    }

    write!(sdl, "{INDENT}{field_name}: {field_type}")?;

    if let Some(default) = &field.default {
        write!(sdl, " = {}", ValueDisplay(default, graph))?;
    }

    write_composed_directives(field.directives, graph, sdl)?;

    sdl.push('\n');
    Ok(())
}

fn write_field(field_id: FieldId, field: &Field, graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    let field_name = &graph[field.name];
    let field_type = render_field_type(&field.r#type, graph);
    let args = render_field_arguments(&graph[field.arguments], graph);

    if let Some(description) = field.description {
        write!(sdl, "{}", Description(&graph[description], INDENT))?;
    }

    write!(sdl, "{INDENT}{field_name}{args}: {field_type}")?;

    for subgraph in &field.resolvable_in {
        write_resolvable_in(*subgraph, field, graph, sdl)?;
    }

    write_provides(field, graph, sdl)?;
    write_requires(field, graph, sdl)?;
    write_composed_directives(field.composed_directives, graph, sdl)?;
    write_overrides(field, graph, sdl)?;
    write_authorized(field_id, graph, sdl)?;

    sdl.push('\n');
    Ok(())
}

fn render_composed_directives(
    directives: Directives,
    f: &mut fmt::Formatter<'_>,
    graph: &FederatedGraph,
) -> fmt::Result {
    for directive in &graph[directives] {
        f.write_str(" ")?;
        write_composed_directive(f, directive, graph)?;
    }

    Ok(())
}

fn write_composed_directives(directives: Directives, graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    with_formatter(sdl, |f| render_composed_directives(directives, f, graph))
}

fn write_resolvable_in(subgraph: SubgraphId, field: &Field, graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    let subgraph_name = GraphEnumVariantName(&graph[graph[subgraph].name]);
    let provides = MaybeDisplay(
        field
            .provides
            .iter()
            .find(|provides| provides.subgraph_id == subgraph)
            .map(|fieldset| format!(", provides: {}", SelectionSetDisplay(&fieldset.fields, graph))),
    );
    let requires = MaybeDisplay(
        field
            .requires
            .iter()
            .find(|requires| requires.subgraph_id == subgraph)
            .map(|fieldset| format!(", requires: {}", SelectionSetDisplay(&fieldset.fields, graph))),
    );
    write!(sdl, " @join__field(graph: {subgraph_name}{provides}{requires})")?;

    Ok(())
}

fn write_overrides(field: &Field, graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    for Override {
        graph: overriding_graph,
        label,
        from,
    } in &field.overrides
    {
        let overrides = match from {
            OverrideSource::Subgraph(subgraph_id) => &graph[graph.subgraphs[subgraph_id.0].name],
            OverrideSource::Missing(string) => &graph[*string],
        };

        let optional_label = if let OverrideLabel::Percent(_) = label {
            format!(", overrideLabel: \"{}\"", label)
        } else {
            String::new()
        };

        let subgraph_name = GraphEnumVariantName(&graph[graph[*overriding_graph].name]);
        write!(
            sdl,
            " @join__field(graph: {subgraph_name}, override: \"{overrides}\"{optional_label})"
        )?;
    }
    Ok(())
}

fn write_provides(field: &Field, graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    for provides in field
        .provides
        .iter()
        .filter(|provide| !field.resolvable_in.contains(&provide.subgraph_id))
    {
        let subgraph_name = GraphEnumVariantName(&graph[graph[provides.subgraph_id].name]);
        let fields = SelectionSetDisplay(&provides.fields, graph);
        write!(sdl, " @join__field(graph: {subgraph_name}, provides: {fields}")?;
    }

    Ok(())
}

fn write_requires(field: &Field, graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    for requires in field
        .requires
        .iter()
        .filter(|requires| !field.resolvable_in.contains(&requires.subgraph_id))
    {
        let subgraph_name = GraphEnumVariantName(&graph[graph[requires.subgraph_id].name]);
        let fields = SelectionSetDisplay(&requires.fields, graph);
        write!(sdl, " @join__field(graph: {subgraph_name}, requires: {fields}")?;
    }

    Ok(())
}

fn write_authorized(field_id: FieldId, graph: &FederatedGraph, sdl: &mut String) -> fmt::Result {
    let start = graph
        .field_authorized_directives
        .partition_point(|(other_field_id, _)| *other_field_id < field_id);

    let directives = graph.field_authorized_directives[start..]
        .iter()
        .take_while(|(other_field_id, _)| *other_field_id == field_id)
        .map(|(_, authorized_directive_id)| &graph[*authorized_directive_id]);

    for directive in directives {
        with_formatter(sdl, |f| render_authorized_directive(directive, f, graph))?;
    }

    Ok(())
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
                let directives = arg.directives;
                let default = arg.default.as_ref();
                (name, r#type, directives, default)
            })
            .peekable();
        let mut out = String::from('(');

        while let Some((name, ty, directives, default)) = inner.next() {
            out.push_str(name);
            out.push_str(": ");
            out.push_str(&ty);

            if let Some(default) = default {
                out.push_str(" = ");
                write!(out, "{}", ValueDisplay(default, graph)).unwrap();
            }

            write_composed_directives(directives, graph, &mut out).unwrap();

            if inner.peek().is_some() {
                out.push_str(", ");
            }
        }
        out.push(')');
        out
    }
}

/// Render an @join__field directive.
fn render_join_field(key: &Key, f: &mut fmt::Formatter<'_>, graph: &FederatedGraph) -> fmt::Result {
    let subgraph_name = GraphEnumVariantName(&graph[graph[key.subgraph_id].name]);

    f.write_str(INDENT)?;

    let mut writer = DirectiveWriter::new("join__type", f, graph)?.arg("graph", subgraph_name)?;

    if !key.fields.is_empty() {
        writer = writer.arg("key", SelectionSetDisplay(&key.fields, graph))?;
    }

    if !key.resolvable {
        writer = writer.arg("resolvable", Value::Boolean(false))?;
    }

    if key.is_interface_object {
        writer = writer.arg("isInterfaceObject", Value::Boolean(true))?;
    }

    drop(writer);

    f.write_str("\n")
}

fn render_join_implement(
    subgraph_id: SubgraphId,
    interface_id: InterfaceId,
    f: &mut fmt::Formatter<'_>,
    graph: &FederatedGraph,
) -> fmt::Result {
    let subgraph_name = GraphEnumVariantName(&graph[graph[subgraph_id].name]);

    f.write_str(INDENT)?;

    DirectiveWriter::new("join__implements", f, graph)?
        .arg("graph", subgraph_name)?
        .arg(
            "interface",
            Value::String(graph.at(interface_id).then(|iface| iface.type_definition_id).name),
        )?;

    Ok(())
}

/// Render an `@authorized` directive
fn render_authorized_directive(
    directive: &AuthorizedDirective,
    f: &mut fmt::Formatter<'_>,
    graph: &FederatedGraph,
) -> fmt::Result {
    f.write_str(" ")?;

    let mut writer = DirectiveWriter::new("authorized", f, graph)?;

    if let Some(fields) = directive.fields.as_ref() {
        writer = writer.arg("fields", SelectionSetDisplay(fields, graph))?;
    }

    if let Some(node) = directive.node.as_ref() {
        writer = writer.arg("node", SelectionSetDisplay(node, graph))?;
    }

    if let Some(arguments) = directive.arguments.as_ref() {
        writer = writer.arg("arguments", InputValueDefinitionSetDisplay(arguments, graph))?;
    }

    if let Some(metadata) = directive.metadata.as_ref() {
        writer.arg("metadata", metadata.clone())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::from_sdl;

    use super::*;

    #[test]
    fn test_render_empty() {
        use expect_test::expect;

        let empty = crate::VersionedFederatedGraph::Sdl(
            crate::render_sdl::render_federated_sdl(&FederatedGraph::default()).unwrap(),
        );

        let actual = render_federated_sdl(&empty.into_latest()).expect("valid");
        let expected = expect![[r#"
            directive @core(feature: String!) repeatable on SCHEMA

            directive @join__owner(graph: join__Graph!) on OBJECT

            directive @join__type(
                graph: join__Graph!
                key: String!
                resolvable: Boolean = true
            ) repeatable on OBJECT | INTERFACE

            directive @join__field(
                graph: join__Graph
                requires: String
                provides: String
            ) on FIELD_DEFINITION

            directive @join__graph(name: String!, url: String!) on ENUM_VALUE

            directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE

            directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION
        "#]];

        expected.assert_eq(&actual);
    }

    #[test]
    fn escape_strings() {
        use expect_test::expect;

        let empty = from_sdl(
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
            directive @core(feature: String!) repeatable on SCHEMA

            directive @join__owner(graph: join__Graph!) on OBJECT

            directive @join__type(
                graph: join__Graph!
                key: String!
                resolvable: Boolean = true
            ) repeatable on OBJECT | INTERFACE

            directive @join__field(
                graph: join__Graph
                requires: String
                provides: String
            ) on FIELD_DEFINITION

            directive @join__graph(name: String!, url: String!) on ENUM_VALUE

            directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE

            directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION

            type Query {
                field: String @deprecated(reason: "This is a \"deprecated\" reason") @dummy(test: "a \"test\"")
            }
        "#]];

        expected.assert_eq(&actual);
    }

    #[test]
    fn multiline_strings() {
        use expect_test::expect;

        let empty = from_sdl(
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
            directive @core(feature: String!) repeatable on SCHEMA

            directive @join__owner(graph: join__Graph!) on OBJECT

            directive @join__type(
                graph: join__Graph!
                key: String!
                resolvable: Boolean = true
            ) repeatable on OBJECT | INTERFACE

            directive @join__field(
                graph: join__Graph
                requires: String
                provides: String
            ) on FIELD_DEFINITION

            directive @join__graph(name: String!, url: String!) on ENUM_VALUE

            directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE

            directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION

            type Query {
                field: String @deprecated(reason: "This is a \"deprecated\" reason\n\n                on multiple lines.\n\n                yes, way\n\n                ") @dummy(test: "a \"test\"")
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

        let parsed = from_sdl(schema).unwrap();
        let rendered = render_federated_sdl(&parsed).unwrap();

        let expected = expect_test::expect![[r#"
            directive @core(feature: String!) repeatable on SCHEMA

            directive @join__owner(graph: join__Graph!) on OBJECT

            directive @join__type(
                graph: join__Graph!
                key: String!
                resolvable: Boolean = true
            ) repeatable on OBJECT | INTERFACE

            directive @join__field(
                graph: join__Graph
                requires: String
                provides: String
            ) on FIELD_DEFINITION

            directive @join__graph(name: String!, url: String!) on ENUM_VALUE

            directive @join__implements(graph: join__Graph!, interface: String!) repeatable on OBJECT | INTERFACE

            directive @join__unionMember(graph: join__Graph!, member: String!) repeatable on UNION

            enum join__Graph {
                MOCKSUBGRAPH @join__graph(name: "mocksubgraph", url: "https://mock.example.com/todo/graphql")
            }



            interface b
                @join__type(graph: MOCKSUBGRAPH)
            {
                c: String @join__field(graph: MOCKSUBGRAPH)
            }
        "#]];

        expected.assert_eq(&rendered);

        // Check that from_sdl accepts the rendered sdl
        {
            from_sdl(&rendered).unwrap();
        }
    }
}
