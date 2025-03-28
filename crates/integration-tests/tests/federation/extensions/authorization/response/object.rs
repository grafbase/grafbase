use engine::Engine;
use graphql_mocks::dynamic::{DynamicSchema, DynamicSubgraph};
use integration_tests::{
    federation::{AuthorizationExt, EngineExt},
    runtime,
};

use crate::federation::extensions::authorization::{deny_some::DenySites, grant_all::GrantAll, user};

fn subgraph() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
        extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

        type Query {
            user: User
            users: [User]
            pets: [Pet]
            node: Node
        }

        interface Node {
            name: String!
        }

        type User implements Node @auth(fields: "name") {
            name: String!
        }

        union Pet = Dog | Cat

        type Dog @auth(fields: "name") {
            name: String!
        }

        type Cat {
            name: String!
        }

        "#,
    )
    .with_resolver("Query", "user", user())
    .with_resolver("Query", "users", serde_json::Value::Array(vec![user(), user()]))
    .with_resolver(
        "Query",
        "pets",
        serde_json::json!([{"__typename": "Dog", "name": "Max"}, {"__typename": "Cat", "name": "Whiskers"}]),
    )
    .with_resolver(
        "Query",
        "node",
        serde_json::json!({"__typename": "User", "name": "Bob"}),
    )
    .into_subgraph("x")
}

#[test]
fn grant_object() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(GrantAll))
            .build()
            .await;

        let response = engine.post("query { user { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "name": "Peter"
            }
          }
        }
        "#);
    });
}

#[test]
fn deny_object_at_query_stage() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::query(["User"])))
            .build()
            .await;

        let response = engine.post("query { user { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": null
          },
          "errors": [
            {
              "message": "Unauthorized at query stage",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "user"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("x");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { __typename @skip(if: true) }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn deny_object_at_response_stage() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::response(["User"])))
            .build()
            .await;

        let response = engine.post("query { user { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": null
          },
          "errors": [
            {
              "message": "Unauthorized at response stage",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "user"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn grant_object_within_list() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(GrantAll))
            .build()
            .await;

        let response = engine.post("query { users { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "users": [
              {
                "name": "Peter"
              },
              {
                "name": "Peter"
              }
            ]
          }
        }
        "#);
    });
}

#[test]
fn deny_object_within_list_at_query_stage() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::query(["User"])))
            .build()
            .await;

        let response = engine.post("query { users { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "users": null
          },
          "errors": [
            {
              "message": "Unauthorized at query stage",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "users"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("x");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { __typename @skip(if: true) }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn deny_object_within_list_at_response_stage() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::response(["User"])))
            .build()
            .await;

        let response = engine.post("query { users { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "users": [
              null,
              null
            ]
          },
          "errors": [
            {
              "message": "Unauthorized at response stage",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "users",
                0
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            },
            {
              "message": "Unauthorized at response stage",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "users",
                1
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn grant_object_within_union() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(GrantAll))
            .build()
            .await;

        let response = engine
            .post("query { pets { ... on Dog { name } ... on Cat { name } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "pets": [
              {
                "name": "Max"
              },
              {
                "name": "Whiskers"
              }
            ]
          }
        }
        "#);
    });
}

#[test]
fn deny_object_within_union_at_query_stage() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::query(["Dog"])))
            .build()
            .await;

        let response = engine
            .post("query { pets { ... on Dog { name } ... on Cat { name } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "pets": [
              null,
              {
                "name": "Whiskers"
              }
            ]
          },
          "errors": [
            {
              "message": "Unauthorized at query stage",
              "locations": [
                {
                  "line": 1,
                  "column": 29
                }
              ],
              "path": [
                "pets",
                0,
                "name"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("x");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { pets { __typename ... on Cat { name } } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn deny_object_within_union_at_response_stage() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::response(["Dog"])))
            .build()
            .await;

        let response = engine
            .post("query { pets { ... on Dog { name } ... on Cat { name } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "pets": [
              null,
              {
                "name": "Whiskers"
              }
            ]
          },
          "errors": [
            {
              "message": "Unauthorized at response stage",
              "locations": [
                {
                  "line": 1,
                  "column": 29
                }
              ],
              "path": [
                "pets",
                0,
                "name"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn grant_object_behind_interface() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(GrantAll))
            .build()
            .await;

        let response = engine.post("query { node { ... on User { name } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": {
              "name": "Bob"
            }
          }
        }
        "#);
    });
}

#[test]
fn deny_object_behind_interface_at_query_stage() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::query(["User"])))
            .build()
            .await;

        let response = engine.post("query { node { ... on User { name } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": null
          },
          "errors": [
            {
              "message": "Unauthorized at query stage",
              "locations": [
                {
                  "line": 1,
                  "column": 30
                }
              ],
              "path": [
                "node",
                "name"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("x");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { node { __typename @skip(if: true) } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn deny_object_behind_interface_at_response_stage() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::response(["User"])))
            .build()
            .await;

        let response = engine.post("query { node { ... on User { name } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": null
          },
          "errors": [
            {
              "message": "Unauthorized at response stage",
              "locations": [
                {
                  "line": 1,
                  "column": 30
                }
              ],
              "path": [
                "node",
                "name"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    });
}
