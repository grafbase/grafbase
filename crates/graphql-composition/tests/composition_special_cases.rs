#![allow(unused_crate_dependencies)]

#[test]
fn subgraph_names_that_differ_only_by_case_are_not_allowed() {
    let mut subgraphs = graphql_composition::Subgraphs::default();

    subgraphs
        .ingest_str("type Query { name: String }", "valid", "example.com")
        .unwrap();

    subgraphs
        .ingest_str("type Query { fullName: String }", "Valid", "example.com")
        .unwrap();

    let result = graphql_composition::compose(&subgraphs);
    let diagnostics = result.diagnostics();
    let messages: Vec<_> = diagnostics.iter_messages().collect();
    assert_eq!(messages.len(), 1);

    assert_eq!(
        messages[0],
        "Found two subgraphs named \"Valid\". Subgraph names are case insensitive."
    );
}
