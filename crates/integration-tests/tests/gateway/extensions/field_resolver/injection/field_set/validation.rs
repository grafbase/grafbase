use crate::gateway::extensions::field_resolver::injection::field_set::{graphql_subgraph, run_with_field_set};

#[test]
fn invalid_selection_set() {
    let err = run_with_field_set(graphql_subgraph(), "{").unwrap_err();
    insta::assert_snapshot!(err, @r#"
    At site User.echo, for the extension 'echo-1.0.0' directive @echo: Could not parse InputValueSet: unexpected open brace ('{') token (expected one of , "..."RawIdent, schema, query, mutation, subscription, ty, input, true, false, null, implements, interface, "enum", union, scalar, extend, directive, repeatable, on, fragment)
    See schema at 23:35:
    (graph: B, extension: ECHO, name: "echo", arguments: {fields: "{"})
    "#);
}
