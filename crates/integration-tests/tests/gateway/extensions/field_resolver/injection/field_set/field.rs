use crate::gateway::extensions::field_resolver::injection::field_set::{graphql_subgraph, run_with_field_set};

#[test]
fn can_select_single_field() {
    let response = run_with_field_set(graphql_subgraph(), "id").unwrap();
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
fn can_select_multiple_fields() {
    let response = run_with_field_set(graphql_subgraph(), "id name address { street } friends { name }").unwrap();
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
                "name": "Peter",
                "address": {
                  "street": "123 Main St"
                },
                "friends": [
                  {
                    "name": "Alice"
                  },
                  {
                    "name": "Bob"
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
fn cannot_select_unknown_fields() {
    let err = run_with_field_set(graphql_subgraph(), "unknown").unwrap_err();
    insta::assert_snapshot!(err, @r#"
    At site User.echo, for the extension 'echo-1.0.0' directive @echo: Unknown field 'unknown' on type 'User'
    See schema at 23:35:
    (graph: B, extension: ECHO, name: "echo", arguments: {fields: "unknown"})
    "#);
}

#[test]
fn cannot_select_unknown_fields_nested() {
    let err = run_with_field_set(graphql_subgraph(), "friends { friends { address { unknown } } }").unwrap_err();
    insta::assert_snapshot!(err, @r#"
    At site User.echo, for the extension 'echo-1.0.0' directive @echo: Unknown field 'unknown' on type 'Address' at path '.friends.friends.address'
    See schema at 23:35:
    (graph: B, extension: ECHO, name: "echo", arguments: {fields: "friends { friends { address { unknown } } }"})
    "#);
}

#[test]
fn composite_type_cannot_be_a_leaf() {
    let err = run_with_field_set(graphql_subgraph(), "id address").unwrap_err();
    insta::assert_snapshot!(err, @r#"
    At site User.echo, for the extension 'echo-1.0.0' directive @echo: Leaf field 'address' must be a scalar or an enum, but is a Address.
    See schema at 23:35:
    (graph: B, extension: ECHO, name: "echo", arguments: {fields: "id address"})
    "#);
}

#[test]
fn composite_type_cannot_be_a_leaf_nested() {
    let err = run_with_field_set(graphql_subgraph(), "id friends { address }").unwrap_err();
    insta::assert_snapshot!(err, @r#"
    At site User.echo, for the extension 'echo-1.0.0' directive @echo: Leaf field 'address' at path '.friends' must be a scalar or an enum, but is a Address.
    See schema at 23:35:
    (graph: B, extension: ECHO, name: "echo", arguments: {fields: "id friends { address }"})
    "#);
}

#[test]
fn scalars_cannot_have_selection_set() {
    let err = run_with_field_set(graphql_subgraph(), "name { __typename }").unwrap_err();
    insta::assert_snapshot!(err, @r#"
    At site User.echo, for the extension 'echo-1.0.0' directive @echo: Field 'name' cannot have a selection set, it's a String!. Only interfaces, unions and objects can.
    See schema at 23:35:
    (graph: B, extension: ECHO, name: "echo", arguments: {fields: "name { __typename }"})
    "#);
}

#[test]
fn scalars_cannot_have_selection_set_nested() {
    let err = run_with_field_set(graphql_subgraph(), "name friends { name { __typename } }").unwrap_err();
    insta::assert_snapshot!(err, @r#"
    At site User.echo, for the extension 'echo-1.0.0' directive @echo: Field 'name' at path '.friends' cannot have a selection set, it's a String!. Only interfaces, unions and objects can.
    See schema at 23:35:
    (graph: B, extension: ECHO, name: "echo", arguments: {fields: "name friends { name { __typename } }"})
    "#);
}
