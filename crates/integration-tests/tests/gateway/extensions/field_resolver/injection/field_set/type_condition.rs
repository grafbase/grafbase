use crate::gateway::extensions::field_resolver::injection::field_set::graphql_subgraph;

use super::run_with_field_set;
use graphql_mocks::dynamic::DynamicSchema;
use serde_json::json;

#[test]
fn can_apply_own_type_as_condition() {
    let response = run_with_field_set(graphql_subgraph(), "... on User { id name }").unwrap();
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "echo": {
            "schema": {},
            "directive": {},
            "input": {
              "fields": {
                "id": "1",
                "name": "Peter"
              }
            }
          }
        }
      }
    }
    "#);
}

#[test]
fn can_have_inline_fragments_without_type_condition() {
    let response = run_with_field_set(graphql_subgraph(), "... { id name }").unwrap();
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "echo": {
            "schema": {},
            "directive": {},
            "input": {
              "fields": {
                "id": "1",
                "name": "Peter"
              }
            }
          }
        }
      }
    }
    "#);
}

#[test]
fn cannot_have_name_fragments() {
    let err = run_with_field_set(graphql_subgraph(), "id ...NamedFragment").unwrap_err();
    insta::assert_snapshot!(err, @r#"
    * At site User.echo, for the extension 'echo-1.0.0' directive @echo: Cannot use named fragments inside a FieldSet
    22 |   age: Int! @join__field(graph: A)
    23 |   echo: JSON @extension__directive(graph: B, extension: ECHO, name: "echo", arguments: {fields: "id ...NamedFragment"}) @join__field(graph: B)
                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    24 |   friends: [User!] @join__field(graph: A)
    "#);
}

#[test]
fn can_select_interface_fields_on_self() {
    let response = run_with_field_set(
        DynamicSchema::builder(
            r#"
        interface Node {
            id: ID!
        }

        type User implements Node {
            id: ID!
            name: String!
        }

        type Query {
            user: User
        }
        "#,
        )
        .with_resolver(
            "Query",
            "user",
            json!({
                "id": "1",
                "name": "Peter",
            }),
        ),
        "... on Node { id }",
    )
    .unwrap();
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "echo": {
            "schema": {},
            "directive": {},
            "input": {
              "fields": {
                "id": "1"
              }
            }
          }
        }
      }
    }
    "#);
}

#[test]
fn type_condiiton_must_have_at_least_one_common_type_union() {
    let err = run_with_field_set(
        DynamicSchema::builder(
            r#"
        type Other {
            id: ID!
        }

        union OtherUnion = Other

        type User {
            id: ID!
            name: String!
        }

        type Query {
            user: User
        }
        "#,
        )
        .with_resolver(
            "Query",
            "user",
            json!({
                "id": "1",
                "name": "Peter",
            }),
        ),
        "... on OtherUnion { ... on Other { id } }",
    )
    .unwrap_err();
    insta::assert_snapshot!(err, @r#"
    * At site User.echo, for the extension 'echo-1.0.0' directive @echo: Type condition on 'OtherUnion' cannot be used in a 'User' selection_set
    26 | {
    27 |   echo: JSON @extension__directive(graph: B, extension: ECHO, name: "echo", arguments: {fields: "... on OtherUnion { ... on Other { id } }"}) @join__field(graph: B)
                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    28 |   id: ID! @join__field(graph: A)
    "#);
}

#[test]
fn type_condiiton_must_have_at_least_one_common_type_interface() {
    let err = run_with_field_set(
        DynamicSchema::builder(
            r#"
        interface Node {
            id: ID!
        }

        type User  {
            id: ID!
            name: String!
        }

        type Query {
            user: User
        }
        "#,
        )
        .with_resolver(
            "Query",
            "user",
            json!({
                "id": "1",
                "name": "Peter",
            }),
        ),
        "... on Node { id }",
    )
    .unwrap_err();
    insta::assert_snapshot!(err, @r#"
    * At site User.echo, for the extension 'echo-1.0.0' directive @echo: Type condition on 'Node' cannot be used in a 'User' selection_set
    20 | {
    21 |   echo: JSON @extension__directive(graph: B, extension: ECHO, name: "echo", arguments: {fields: "... on Node { id }"}) @join__field(graph: B)
                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    22 |   id: ID! @join__field(graph: A)
    "#);
}

#[test]
fn can_select_interface_fields_nested() {
    let response = run_with_field_set(
        DynamicSchema::builder(
            r#"
        interface Node {
            id: ID!
        }

        type User implements Node {
            id: ID!
            name: String!
            nodes: [Node!]
        }

        type Something implements Node {
            id: ID!
        }

        type Query {
            user: User
        }
        "#,
        )
        .with_resolver(
            "Query",
            "user",
            json!({
                "id": "1",
                "name": "Peter",
                "nodes": [
                    {"__typename": "Something", "id": "2"},
                    {"__typename": "User", "id": "3"},
                ]
            }),
        ),
        "nodes { ... on Node { id } }",
    )
    .unwrap();
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "echo": {
            "schema": {},
            "directive": {},
            "input": {
              "fields": {
                "nodes": [
                  {
                    "id": "2"
                  },
                  {
                    "id": "3"
                  }
                ]
              }
            }
          }
        }
      }
    }
    "#);
}

#[test]
fn can_apply_object_type_condition_nested() {
    let response = run_with_field_set(
        DynamicSchema::builder(
            r#"
        interface Node {
            id: ID!
        }

        type User implements Node {
            id: ID!
            name: String!
            nodes: [Node!]
        }

        type Something implements Node {
            id: ID!
        }

        type Query {
            user: User
        }
        "#,
        )
        .with_resolver(
            "Query",
            "user",
            json!({
                "id": "1",
                "name": "Peter",
                "nodes": [
                    {"__typename": "Something", "id": "2"},
                    {"__typename": "User", "id": "3"},
                ]
            }),
        ),
        "nodes { ... on Something { id } }",
    )
    .unwrap();
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "echo": {
            "schema": {},
            "directive": {},
            "input": {
              "fields": {
                "nodes": [
                  {
                    "id": "2"
                  },
                  {}
                ]
              }
            }
          }
        }
      }
    }
    "#);
}

#[test]
fn can_select_fields_from_union() {
    let response = run_with_field_set(graphql_subgraph(), "pets { ... on Cat { name } }").unwrap();
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "echo": {
            "schema": {},
            "directive": {},
            "input": {
              "fields": {
                "pets": [
                  {},
                  {
                    "name": "Whiskers"
                  }
                ]
              }
            }
          }
        }
      }
    }
    "#);
}

#[test]
fn unions_have_no_fields() {
    let error = run_with_field_set(graphql_subgraph(), "pets { id }").unwrap_err();
    insta::assert_snapshot!(error, @r#"
    * At site User.echo, for the extension 'echo-1.0.0' directive @echo: Field 'id' at path '.pets' does not exists on Pet, it's a union. Only interfaces and objects have fields, consider using a fragment with a type condition.
    22 |   age: Int! @join__field(graph: A)
    23 |   echo: JSON @extension__directive(graph: B, extension: ECHO, name: "echo", arguments: {fields: "pets { id }"}) @join__field(graph: B)
                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    24 |   friends: [User!] @join__field(graph: A)
    "#);
}

#[test]
fn inexsitant_type() {
    let error = run_with_field_set(graphql_subgraph(), "... on Unknown { id }").unwrap_err();
    insta::assert_snapshot!(error, @r#"
    * At site User.echo, for the extension 'echo-1.0.0' directive @echo: Unknown type 'Unknown'
    22 |   age: Int! @join__field(graph: A)
    23 |   echo: JSON @extension__directive(graph: B, extension: ECHO, name: "echo", arguments: {fields: "... on Unknown { id }"}) @join__field(graph: B)
                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    24 |   friends: [User!] @join__field(graph: A)
    "#);
}

#[test]
fn must_be_output_type() {
    let error = run_with_field_set(
        DynamicSchema::builder(
            r#"
        type Query {
            user: User
        }

        type User {
            id: ID!
        }

        input UserInput {
            id: ID!
        }
        "#,
        ),
        "... on UserInput { id }",
    )
    .unwrap_err();
    insta::assert_snapshot!(error, @r#"
    * At site User.echo, for the extension 'echo-1.0.0' directive @echo: UserInput is not an object, interface or union
    20 | {
    21 |   echo: JSON @extension__directive(graph: B, extension: ECHO, name: "echo", arguments: {fields: "... on UserInput { id }"}) @join__field(graph: B)
                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    22 |   id: ID! @join__field(graph: A)
    "#);
}
