use engine::Engine;
use graphql_mocks::dynamic::{DynamicSchema, DynamicSubgraph};
use integration_tests::{federation::EngineExt, runtime};

use crate::federation::extensions::authorization::{AuthorizationExt, injection::EchoInjections, user};

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
            id: ID!
            name: String!
        }

        type User implements Node @auth(input: {bestfriend: "dog"}, fields: "name") {
            id: ID!
            name: String!
        }

        union Pet = Dog | Cat

        type Dog @auth(input: {bestfriend: "dog"}, fields: "name") {
            id: ID!
            name: String!
        }

        type Cat {
            id: ID!
            name: String!
        }

        "#,
    )
    .with_resolver("Query", "user", user())
    .with_resolver(
        "Query",
        "users",
        serde_json::json!([
            {
                "id": "1",
                "name": "Peter",
            },
            {
                "id": "2",
                "name": "Alice",
            }

        ]),
    )
    .with_resolver(
        "Query",
        "pets",
        serde_json::json!([{"__typename": "Dog", "name": "Max"}, {"__typename": "Cat", "name": "Whiskers"}]),
    )
    .with_resolver(
        "Query",
        "node",
        serde_json::json!({"__typename": "User", "id": "80", "name": "Bob"}),
    )
    .into_subgraph("x")
}

#[test]
fn object() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(EchoInjections))
            .build()
            .await;

        let response = engine.post("query { user { id } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": null
          },
          "errors": [
            {
              "message": "Injection time!",
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
                "injections": {
                  "query": {
                    "auth": {
                      "User": {
                        "input": {
                          "bestfriend": "dog"
                        }
                      }
                    }
                  },
                  "response": {
                    "directive_name": "auth",
                    "directive_site": "User",
                    "items": [
                      {
                        "fields": {
                          "name": "Peter"
                        }
                      }
                    ]
                  }
                },
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn object_within_list() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(EchoInjections))
            .build()
            .await;

        let response = engine.post("query { users { id } }").await;
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
              "message": "Injection time!",
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
                "injections": {
                  "query": {
                    "auth": {
                      "User": {
                        "input": {
                          "bestfriend": "dog"
                        }
                      }
                    }
                  },
                  "response": {
                    "directive_name": "auth",
                    "directive_site": "User",
                    "items": [
                      {
                        "fields": {
                          "name": "Peter"
                        }
                      },
                      {
                        "fields": {
                          "name": "Alice"
                        }
                      }
                    ]
                  }
                },
                "code": "UNAUTHORIZED"
              }
            },
            {
              "message": "Injection time!",
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
                "injections": {
                  "query": {
                    "auth": {
                      "User": {
                        "input": {
                          "bestfriend": "dog"
                        }
                      }
                    }
                  },
                  "response": {
                    "directive_name": "auth",
                    "directive_site": "User",
                    "items": [
                      {
                        "fields": {
                          "name": "Peter"
                        }
                      },
                      {
                        "fields": {
                          "name": "Alice"
                        }
                      }
                    ]
                  }
                },
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn object_within_union() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(EchoInjections))
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
              "message": "Injection time!",
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
                "injections": {
                  "query": {
                    "auth": {
                      "Dog": {
                        "input": {
                          "bestfriend": "dog"
                        }
                      }
                    }
                  },
                  "response": {
                    "directive_name": "auth",
                    "directive_site": "Dog",
                    "items": [
                      {
                        "fields": {
                          "name": "Max"
                        }
                      }
                    ]
                  }
                },
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn object_behind_interface() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(EchoInjections))
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
              "message": "Injection time!",
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
                "injections": {
                  "query": {
                    "auth": {
                      "User": {
                        "input": {
                          "bestfriend": "dog"
                        }
                      }
                    }
                  },
                  "response": {
                    "directive_name": "auth",
                    "directive_site": "User",
                    "items": [
                      {
                        "fields": {
                          "name": "Bob"
                        }
                      }
                    ]
                  }
                },
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    });
}
