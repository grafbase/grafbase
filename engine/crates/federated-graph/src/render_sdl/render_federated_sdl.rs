use super::display_utils::*;
use crate::{federated_graph::*, FederatedGraphV3};
use std::fmt::{self, Display, Write};

/// Render a GraphQL SDL string for a federated graph. It includes [join spec
/// directives](https://specs.apollo.dev/join/v0.3/) about subgraphs and entities.
pub fn render_federated_sdl(graph: &FederatedGraphV3) -> Result<String, fmt::Error> {
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

    for (object_id, object) in graph.iter_objects() {
        let object_name = &graph[object.name];

        let mut fields = graph[object.fields.clone()]
            .iter()
            .enumerate()
            .filter(|(_idx, field)| !graph[field.name].starts_with("__"))
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
                let interface_name = &graph[graph[*interface].name];
                sdl.push_str(interface_name);

                if idx < object.implements_interfaces.len() - 1 {
                    sdl.push_str(" & ");
                }
            }
        }

        write_composed_directives(object.composed_directives, graph, &mut sdl)?;

        for authorized_directive in graph.object_authorized_directives(object_id) {
            write!(sdl, "{}", AuthorizedDirectiveDisplay(authorized_directive, graph))?;
        }

        if !object.keys.is_empty() {
            sdl.push('\n');
            for key in &object.keys {
                let subgraph_name = GraphEnumVariantName(&graph[graph[key.subgraph_id].name]);
                if key.fields.is_empty() {
                    writeln!(
                        sdl,
                        r#"{INDENT}@join__type(graph: {subgraph_name}{resolvable})"#,
                        resolvable = if key.resolvable { "" } else { ", resolvable: false" }
                    )?;
                } else {
                    writeln!(
                        sdl,
                        r#"{INDENT}@join__type(graph: {subgraph_name}, key: {selection_set}{resolvable})"#,
                        selection_set = FieldSetDisplay(&key.fields, graph),
                        resolvable = if key.resolvable { "" } else { ", resolvable: false" }
                    )?;
                }
            }
        } else {
            sdl.push(' ');
        }

        sdl.push_str("{\n");

        for (idx, field) in fields {
            let field_id = FieldId(object.fields.start.0 + idx);
            write_field(field_id, field, graph, &mut sdl)?;
        }

        writeln!(sdl, "}}\n")?;
    }

    for (interface_id, interface) in graph.iter_interfaces() {
        let interface_name = &graph[interface.name];

        if let Some(description) = interface.description {
            write!(sdl, "{}", Description(&graph[description], ""))?;
        }

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

        for authorized_directive in graph.interface_authorized_directives(interface_id) {
            write!(sdl, "{}", AuthorizedDirectiveDisplay(authorized_directive, graph))?;
        }

        write_composed_directives(interface.composed_directives, graph, &mut sdl)?;

        if interface.keys.is_empty() {
            sdl.push_str(" {\n");
        } else {
            sdl.push('\n');
            for resolvable_key in &interface.keys {
                let selection_set = FieldSetDisplay(&resolvable_key.fields, graph);
                let subgraph_name = GraphEnumVariantName(&graph[graph[resolvable_key.subgraph_id].name]);
                let is_interface_object = if resolvable_key.is_interface_object {
                    ", isInterfaceObject: true"
                } else {
                    ""
                };
                writeln!(
                    sdl,
                    r#"{INDENT}@join__type(graph: {subgraph_name}, key: {selection_set}{is_interface_object})"#
                )?;
            }

            sdl.push_str("{\n");
        }

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
        sdl.push_str(" = ");

        let mut members = union.members.iter().peekable();

        while let Some(member) = members.next() {
            sdl.push_str(&graph[graph[*member].name]);

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
    "#});

    sdl.push('\n');
    Ok(())
}

fn write_subgraphs_enum(graph: &FederatedGraphV3, sdl: &mut String) -> fmt::Result {
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

fn write_input_field(field: &InputValueDefinition, graph: &FederatedGraphV3, sdl: &mut String) -> fmt::Result {
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

fn write_field(field_id: FieldId, field: &Field, graph: &FederatedGraphV3, sdl: &mut String) -> fmt::Result {
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

fn write_composed_directives(directives: Directives, graph: &FederatedGraphV3, sdl: &mut String) -> fmt::Result {
    for directive in &graph[directives] {
        write!(sdl, "{}", DirectiveDisplay(directive, graph))?;
    }

    Ok(())
}

fn write_resolvable_in(subgraph: SubgraphId, field: &Field, graph: &FederatedGraphV3, sdl: &mut String) -> fmt::Result {
    let subgraph_name = GraphEnumVariantName(&graph[graph[subgraph].name]);
    let provides = MaybeDisplay(
        field
            .provides
            .iter()
            .find(|provides| provides.subgraph_id == subgraph)
            .map(|fieldset| format!(", provides: {}", FieldSetDisplay(&fieldset.fields, graph))),
    );
    let requires = MaybeDisplay(
        field
            .requires
            .iter()
            .find(|requires| requires.subgraph_id == subgraph)
            .map(|fieldset| format!(", requires: {}", FieldSetDisplay(&fieldset.fields, graph))),
    );
    write!(sdl, " @join__field(graph: {subgraph_name}{provides}{requires})")?;

    Ok(())
}

fn write_overrides(field: &Field, graph: &FederatedGraphV3, sdl: &mut String) -> fmt::Result {
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

fn write_provides(field: &Field, graph: &FederatedGraphV3, sdl: &mut String) -> fmt::Result {
    for provides in field
        .provides
        .iter()
        .filter(|provide| !field.resolvable_in.contains(&provide.subgraph_id))
    {
        let subgraph_name = GraphEnumVariantName(&graph[graph[provides.subgraph_id].name]);
        let fields = FieldSetDisplay(&provides.fields, graph);
        write!(sdl, " @join__field(graph: {subgraph_name}, provides: {fields}")?;
    }

    Ok(())
}

fn write_requires(field: &Field, graph: &FederatedGraphV3, sdl: &mut String) -> fmt::Result {
    for requires in field
        .requires
        .iter()
        .filter(|requires| !field.resolvable_in.contains(&requires.subgraph_id))
    {
        let subgraph_name = GraphEnumVariantName(&graph[graph[requires.subgraph_id].name]);
        let fields = FieldSetDisplay(&requires.fields, graph);
        write!(sdl, " @join__field(graph: {subgraph_name}, requires: {fields}")?;
    }

    Ok(())
}

fn write_authorized(field_id: FieldId, graph: &FederatedGraphV3, sdl: &mut String) -> fmt::Result {
    let start = graph
        .field_authorized_directives
        .partition_point(|(other_field_id, _)| *other_field_id < field_id);

    let directives = graph.field_authorized_directives[start..]
        .iter()
        .take_while(|(other_field_id, _)| *other_field_id == field_id)
        .map(|(_, authorized_directive_id)| &graph[*authorized_directive_id]);

    for directive in directives {
        write!(sdl, "{}", AuthorizedDirectiveDisplay(directive, graph))?;
    }

    Ok(())
}

fn render_field_arguments(args: &[InputValueDefinition], graph: &FederatedGraphV3) -> String {
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

struct GraphEnumVariantName<'a>(&'a str);

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

#[cfg(test)]
mod tests {
    use crate::from_sdl;

    use super::*;

    #[test]
    fn test_render_empty() {
        use expect_test::expect;

        let empty = crate::FederatedGraph::V3(FederatedGraphV3::default());
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

            type Query {
                field: String @deprecated(reason: "This is a \"deprecated\" reason\n\non multiple lines.\n\nyes, way") @dummy(test: "a \"test\"")
            }
        "#]];

        expected.assert_eq(&actual);
    }
}
