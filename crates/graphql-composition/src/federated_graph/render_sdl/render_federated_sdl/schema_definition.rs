use super::*;

pub(super) fn display_schema_definition(graph: &FederatedGraph, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let query_root_defined = graph.roots.query.is_some_and(|root| !graph[root].fields.is_empty());

    if !query_root_defined {
        f.write_str("extend ")?;
    }

    f.write_str("schema\n")?;

    f.write_str(INDENT)?;
    DirectiveWriter::new("link", f, graph)?.arg("url", "https://specs.apollo.dev/link/v1.0")?;
    f.write_str("\n")?;

    f.write_str(INDENT)?;
    DirectiveWriter::new("link", f, graph)?
        .arg("url", "https://specs.apollo.dev/join/v0.3")?
        .arg("for", AnyValue::EnumValue("EXECUTION"))?;
    f.write_str("\n")?;

    f.write_str(INDENT)?;
    DirectiveWriter::new("link", f, graph)?
        .arg("url", "https://specs.apollo.dev/inaccessible/v0.2")?
        .arg("for", AnyValue::EnumValue("SECURITY"))?;
    f.write_str("\n")?;

    for linked_schema in &graph.linked_schemas {
        f.write_str(INDENT)?;
        DirectiveWriter::new("link", f, graph)?
            .arg("url", Value::String(linked_schema.url))?
            .arg(
                "import",
                AnyValue::List(
                    linked_schema
                        .imports
                        .iter()
                        .map(|import| AnyValue::from(format!("@{}", graph[*import])))
                        .collect::<Vec<_>>(),
                ),
            )?;
        f.write_str("\n")?;
    }

    if !query_root_defined {
        return f.write_str("\n");
    }

    f.write_str("{\n")?;

    if let Some(query) = graph.roots.query {
        f.write_str(INDENT)?;
        f.write_str("query: ")?;
        f.write_str(&graph[graph[query].name])?;
        f.write_str("\n")?;
    }

    if let Some(mutation) = graph.roots.mutation {
        f.write_str(INDENT)?;
        f.write_str("mutation: ")?;
        f.write_str(&graph[graph[mutation].name])?;
        f.write_str("\n")?;
    }

    if let Some(subscription) = graph.roots.subscription {
        f.write_str(INDENT)?;
        f.write_str("subscription: ")?;
        f.write_str(&graph[graph[subscription].name])?;
        f.write_str("\n")?;
    }

    f.write_str("}\n\n")
}
