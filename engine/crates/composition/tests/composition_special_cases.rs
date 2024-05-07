#![allow(unused_crate_dependencies)]

#[test]
fn subgraph_names_that_differ_only_by_case_are_not_allowed() {
    let mut subgraphs = graphql_composition::Subgraphs::default();

    {
        let schema = async_graphql_parser::parse_schema("type Query { name: String }").unwrap();
        subgraphs.ingest(&schema, "valid", "example.com");
    }

    {
        let schema = async_graphql_parser::parse_schema("type Query { fullName: String }").unwrap();
        subgraphs.ingest(&schema, "Valid", "example.com");
    }

    let result = graphql_composition::compose(&subgraphs);
    let diagnostics = result.diagnostics();
    let messages: Vec<_> = diagnostics.iter_messages().collect();
    assert_eq!(messages.len(), 1);

    assert_eq!(
        messages[0],
        "Found two subgraphs named \"Valid\". Subgraph names are case insensitive."
    );
}
