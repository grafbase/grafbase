use engine::Engine;
use graphql_mocks::{
    FakeGithubSchema, FederatedAccountsSchema, FederatedInventorySchema, FederatedProductsSchema,
    FederatedReviewsSchema, FederatedShippingSchema,
};
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn grafbase_extension_on_successful_request() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .post("query { serverVersion }")
            .header("x-grafbase-telemetry", "yes")
            .await;

        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "serverVersion": "1"
          },
          "extensions": {
            "grafbase": {
              "traceId": "0",
              "queryPlan": {
                "nodes": [
                  {
                    "type": "graphql",
                    "subgraph_name": "github",
                    "request": {
                      "query": "query { serverVersion }"
                    }
                  }
                ],
                "edges": []
              }
            }
          }
        }
        "#
        );
    })
}

#[test]
fn dot_not_include_query_plan() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
            [telemetry.exporters.response_extension]
            query_plan = false
            "#,
            )
            .build()
            .await;

        let response = engine
            .post("query { serverVersion }")
            .header("x-grafbase-telemetry", "yes")
            .await;

        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "serverVersion": "1"
          },
          "extensions": {
            "grafbase": {
              "traceId": "0"
            }
          }
        }
        "#
        );
    })
}

#[test]
fn dot_not_include_trace_id() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
            [telemetry.exporters.response_extension]
            trace_id = false
            "#,
            )
            .build()
            .await;

        let response = engine
            .post("query { serverVersion }")
            .header("x-grafbase-telemetry", "yes")
            .await;

        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "serverVersion": "1"
          },
          "extensions": {
            "grafbase": {
              "queryPlan": {
                "nodes": [
                  {
                    "type": "graphql",
                    "subgraph_name": "github",
                    "request": {
                      "query": "query { serverVersion }"
                    }
                  }
                ],
                "edges": []
              }
            }
          }
        }
        "#
        );
    })
}

#[test]
fn grafbase_extension_on_invalid_request() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FakeGithubSchema)
            .build()
            .await;

        let response = engine
            .post("query x }")
            .header("x-grafbase-telemetry", "yes")
            .await;

        insta::assert_json_snapshot!(
            response,
            @r#"
            {
              "errors": [
                {
                  "message": " --> 1:9\n  |\n1 | query x }\n  |         ^---\n  |\n  = expected variable_definitions, selection_set, or directive",
                  "locations": [
                    {
                      "line": 1,
                      "column": 9
                    }
                  ],
                  "extensions": {
                    "code": "OPERATION_PARSING_ERROR"
                  }
                }
              ],
              "extensions": {
                "grafbase": {
                  "traceId": "0"
                }
              }
            }
            "#
        );
    })
}

#[test]
fn grafbase_extension_secret_value() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
            [[telemetry.exporters.response_extension.access_control]]
            rule = "header"
            name = "dummy"
            value = "secret"
            "#,
            )
            .build()
            .await;

        let response = engine
            .post("query { serverVersion }")
            // shouldn't work anymore
            .header("x-grafbase-telemetry", "yes")
            .await;

        insta::assert_json_snapshot!(
            response,
            @r#"
            {
              "data": {
                "serverVersion": "1"
              }
            }
            "#
        );

        let response = engine
            .post("query { serverVersion }")
            // not the right value
            .header("dummy", "yes")
            .await;

        insta::assert_json_snapshot!(
            response,
            @r#"
            {
              "data": {
                "serverVersion": "1"
              }
            }
            "#
        );
        let response = engine.post("query { serverVersion }").header("dummy", "secret").await;

        insta::assert_json_snapshot!(
            response,
            @r#"
        {
          "data": {
            "serverVersion": "1"
          },
          "extensions": {
            "grafbase": {
              "traceId": "0",
              "queryPlan": {
                "nodes": [
                  {
                    "type": "graphql",
                    "subgraph_name": "github",
                    "request": {
                      "query": "query { serverVersion }"
                    }
                  }
                ],
                "edges": []
              }
            }
          }
        }
        "#
        );
    })
}

#[test]
fn grafbase_extension_denied() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
            [[telemetry.exporters.response_extension.access_control]]
            rule = "deny"
            "#,
            )
            .build()
            .await;

        let response = engine
            .post("query { serverVersion }")
            // shouldn't work anymore
            .header("x-grafbase-telemetry", "yes")
            .await;

        insta::assert_json_snapshot!(
            response,
            @r#"
            {
              "data": {
                "serverVersion": "1"
              }
            }
            "#
        );
    })
}

#[test]
fn grafbase_extension_on_ill_formed_graphql_over_http_request() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::POST)
                    .header(
                        http::HeaderName::from_static("x-grafbase-telemetry"),
                        http::HeaderValue::from_static(""),
                    )
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .header(http::header::ACCEPT, "application/graphql-response+json")
                    .body(Vec::from(br###"{}"###))
                    .unwrap(),
            )
            .await;
        let status = response.status();
        let body: serde_json::Value = serde_json::from_slice(&response.into_body()).unwrap();
        insta::assert_json_snapshot!(body, @r#"
        {
          "errors": [
            {
              "message": "Missing query",
              "extensions": {
                "code": "BAD_REQUEST"
              }
            }
          ],
          "extensions": {
            "grafbase": {
              "traceId": "0"
            }
          }
        }
        "#);
        assert_eq!(status, 400);
    })
}

#[test]
fn complex_query_plan() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedAccountsSchema)
            .with_subgraph(FederatedProductsSchema)
            .with_subgraph(FederatedReviewsSchema)
            .with_subgraph(FederatedInventorySchema)
            .with_subgraph(FederatedShippingSchema)
            .with_toml_config(
                r#"
                [graph]
                introspection = true
                "#)
            .build()
            .await;
        let response = engine
            .post(
                r#"
                query {
                  __schema {
                    queryType { name }
                  }
                  me {
                    id
                    username
                    cart {
                      products {
                        availableShippingService {
                          __typename
                          name
                          reviews {
                            body
                          }
                        }
                        price
                        reviews {
                          author {
                            id
                            username
                          }
                          body
                        }
                      }
                    }
                  }
                }
            "#,
            )
            .header("x-grafbase-telemetry", "")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "__schema": {
              "queryType": {
                "name": "Query"
              }
            },
            "me": {
              "id": "1234",
              "username": "Me",
              "cart": {
                "products": [
                  {
                    "availableShippingService": [
                      {
                        "__typename": "DeliveryCompany",
                        "name": "Planet Express",
                        "reviews": [
                          {
                            "body": "Not as good as Mom's Friendly Delivery Company"
                          }
                        ]
                      }
                    ],
                    "price": 22,
                    "reviews": [
                      {
                        "author": {
                          "id": "1234",
                          "username": "Me"
                        },
                        "body": "Fedoras are one of the most fashionable hats around and can look great with a variety of outfits."
                      }
                    ]
                  },
                  {
                    "availableShippingService": [
                      {
                        "__typename": "DeliveryCompany",
                        "name": "Planet Express",
                        "reviews": [
                          {
                            "body": "Not as good as Mom's Friendly Delivery Company"
                          }
                        ]
                      }
                    ],
                    "price": 55,
                    "reviews": [
                      {
                        "author": null,
                        "body": "Beautiful Pink, my parrot loves it. Definitely recommend!"
                      }
                    ]
                  }
                ]
              }
            }
          },
          "extensions": {
            "grafbase": {
              "traceId": "0",
              "queryPlan": {
                "nodes": [
                  {
                    "type": "graphql",
                    "subgraph_name": "accounts",
                    "request": {
                      "query": "query { me { id username cart { products { name } } } }"
                    }
                  },
                  {
                    "type": "graphql",
                    "subgraph_name": "reviews",
                    "request": {
                      "query": "query($var0: [_Any!]!) { _entities(representations: $var0) { ... on Product { reviews { author { id } body } } } }"
                    }
                  },
                  {
                    "type": "graphql",
                    "subgraph_name": "accounts",
                    "request": {
                      "query": "query($var0: [_Any!]!) { _entities(representations: $var0) { ... on User { username } } }"
                    }
                  },
                  {
                    "type": "graphql",
                    "subgraph_name": "products",
                    "request": {
                      "query": "query($var0: [_Any!]!) { _entities(representations: $var0) { ... on Product { price upc weight(unit: KILOGRAM) } } }"
                    }
                  },
                  {
                    "type": "graphql",
                    "subgraph_name": "inventory",
                    "request": {
                      "query": "query($var0: [_Any!]!) { _entities(representations: $var0) { ... on Product { availableShippingService { __typename name id } } } }"
                    }
                  },
                  {
                    "type": "graphql",
                    "subgraph_name": "reviews",
                    "request": {
                      "query": "query($var0: [_Any!]!) { _entities(representations: $var0) { ... on ShippingService { reviews { body } } } }"
                    }
                  },
                  {
                    "type": "introspection"
                  }
                ],
                "edges": [
                  [
                    0,
                    3
                  ],
                  [
                    1,
                    2
                  ],
                  [
                    3,
                    1
                  ],
                  [
                    3,
                    4
                  ],
                  [
                    4,
                    5
                  ]
                ]
              }
            }
          }
        }
        "#);
    })
}
