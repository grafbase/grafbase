mod any;
mod boolean;
mod entities_list;
mod float;
mod int;
mod list;
mod object;
mod polymorphic;
mod string;

use integration_tests::{
    gateway::{DeterministicEngine, GraphqlResponse},
    runtime,
};
use serde_json::json;

fn run(schema: &str, query: &str, subgraph_response: serde_json::Value) -> GraphqlResponse {
    let schema = [
        r#"
        directive @core(feature: String!) repeatable on SCHEMA

        directive @join__owner(graph: join__Graph!) on OBJECT

        directive @join__type(graph: join__Graph!, key: String!, resolvable: Boolean = true) repeatable on OBJECT | INTERFACE

        directive @join__field(graph: join__Graph, requires: String, provides: String) on FIELD_DEFINITION

        directive @join__graph(name: String!, url: String!) on ENUM_VALUE

        enum join__Graph {
          A @join__graph(name: "accounts", url: "http://127.0.0.1:46697")
        }
        "#,
        schema,
    ]
    .join("\n");
    runtime().block_on(async {
        DeterministicEngine::new(&schema, query, &[subgraph_response])
            .await
            .execute()
            .await
    })
}

#[test]
fn invalid_int() {
    let response = run(
        r#"
        type Query {
            user: User @join__field(graph: A)
        }
        type User {
            age: Int @join__field(graph: A)
        }
        "#,
        r#"
        {
            user {
               age 
            }
        }
        "#,
        json!({"data": {"user": {"age": "??"}}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "age": null
        }
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 4,
              "column": 16
            }
          ],
          "path": [
            "user",
            "age"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn invalid_int2() {
    let response = run(
        r#"
        type Query {
            user: User @join__field(graph: A)
        }
        type User {
            age: Int @join__field(graph: A)
        }
        "#,
        r#"
        {
            user {
               age 
            }
        }
        "#,
        json!({"data": {"user": {"age": []}}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "user": {
          "age": null
        }
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 4,
              "column": 16
            }
          ],
          "path": [
            "user",
            "age"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn invalid_nullable_field_in_list() {
    let response = run(
        r#"
        type Query {
            users: [User] @join__field(graph: A)
        }
        type User {
            username: String @join__field(graph: A)
            age: Int @join__field(graph: A)
        }
        "#,
        r#"
        {
            users {
                username
                age
            }
        }
        "#,
        json!({"data": {"users": [{"username": "Bob", "age": 19}, {"username": "Alice", "age": "??"}]}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": [
          {
            "username": "Bob",
            "age": 19
          },
          {
            "username": "Alice",
            "age": null
          }
        ]
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 5,
              "column": 17
            }
          ],
          "path": [
            "users",
            1,
            "age"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn invalid_required_field_in_list() {
    let response = run(
        r#"
        type Query {
            users: [User] @join__field(graph: A)
        }
        type User {
            username: String @join__field(graph: A)
            age: Int! @join__field(graph: A)
        }
        "#,
        r#"
        {
            users {
                username
                age
            }
        }
        "#,
        json!({"data": {"users": [{"username": "Bob", "age": 19}, {"username": "Alice", "age": "??"}]}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": [
          {
            "username": "Bob",
            "age": 19
          },
          null
        ]
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 5,
              "column": 17
            }
          ],
          "path": [
            "users",
            1,
            "age"
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn invalid_nullable_object_in_list() {
    let response = run(
        r#"
        type Query {
            users: [User] @join__field(graph: A)
        }
        type User {
            username: String @join__field(graph: A)
            age: Int! @join__field(graph: A)
        }
        "#,
        r#"
        {
            users {
                username
                age
            }
        }
        "#,
        json!({"data": {"users": [{"username": "Bob", "age": 19}, []]}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": [
          {
            "username": "Bob",
            "age": 19
          },
          null
        ]
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 3,
              "column": 13
            }
          ],
          "path": [
            "users",
            1
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn invalid_required_object_in_list() {
    let response = run(
        r#"
        type Query {
            users: [User!] @join__field(graph: A)
        }
        type User {
            username: String @join__field(graph: A)
            age: Int! @join__field(graph: A)
        }
        "#,
        r#"
        {
            users {
                username
                age
            }
        }
        "#,
        json!({"data": {"users": [{"username": "Bob", "age": 19}, []]}}),
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "users": null
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 3,
              "column": 13
            }
          ],
          "path": [
            "users",
            1
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}
