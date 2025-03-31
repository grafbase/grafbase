use crate::federation::extensions::field_resolver::injection::field_set::graphql_subgraph;

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
    let error = run_with_field_set(graphql_subgraph(), "id ...NamedFragment").err();
    insta::assert_debug_snapshot!(error, @r#"
    Some(
        "At User.echo for the extension 'echo-1.0.0' directive @echo: Cannot use named fragments inside a FieldSet",
    )
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
    .err();
    insta::assert_debug_snapshot!(err, @r#"
    Some(
        "At User.echo for the extension 'echo-1.0.0' directive @echo: Type condition on 'OtherUnion' cannot be used in a 'User' selection_set",
    )
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
    .err();
    insta::assert_debug_snapshot!(err, @r#"
    Some(
        "At User.echo for the extension 'echo-1.0.0' directive @echo: Type condition on 'Node' cannot be used in a 'User' selection_set",
    )
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
    let error = run_with_field_set(graphql_subgraph(), "pets { id }").err();
    insta::assert_debug_snapshot!(error, @r#"
    Some(
        "At User.echo for the extension 'echo-1.0.0' directive @echo: Field 'id' at path '.pets' does not exists on Pet, it's a union. Only interfaces and objects have fields, consider using a fragment with a type condition.",
    )
    "#);
}

#[test]
fn inexsitant_type() {
    let error = run_with_field_set(graphql_subgraph(), "... on Unknown { id }").err();
    insta::assert_debug_snapshot!(error, @r#"
    Some(
        "At User.echo for the extension 'echo-1.0.0' directive @echo: Uknown type 'Unknown'",
    )
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
    .err();
    insta::assert_debug_snapshot!(error, @r#"
    Some(
        "At User.echo for the extension 'echo-1.0.0' directive @echo: UserInput is not an object, interface or union",
    )
    "#);
}
