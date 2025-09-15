use crate::gateway::extensions::field_resolver::injection::field_set::{graphql_subgraph, run_with_field_set};

#[test]
fn invalid_selection_set() {
    let err = run_with_field_set(graphql_subgraph(), "{").unwrap_err();
    insta::assert_snapshot!(err, @r#"
    * At site User.echo, for the extension 'echo-1.0.0' directive @echo: Could not parse InputValueSet: unexpected open brace ('{') token (expected one of , "..."RawIdent, schema, query, mutation, subscription, ty, input, true, false, null, implements, interface, "enum", union, scalar, extend, directive, repeatable, on, fragment)
    30 |   age: Int! @join__field(graph: A)
    31 |   echo: JSON @extension__directive(graph: B, extension: ECHO, name: "echo", arguments: {fields: "{"}) @join__field(graph: B)
                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    32 |   friends: [User!] @join__field(graph: A)
    "#);
}
