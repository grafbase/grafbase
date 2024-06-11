//! General tests of GraphQL execution in Grafbase engine
//!
//! I wanted to call this module `graphql` but decided that could be confusing with the
//! GraphQL engine.
//!
//! A lot of these make use of OpenAPI at the moment but only because it's easy to
//! set up an OpenAPI connector.  There's no real reason they need to.

use std::net::SocketAddr;

mod interfaces;
mod joins;
mod requires;
mod unions;

use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{runtime, udfs::RustUdfs, Engine, EngineBuilder, ResponseExt};
use runtime::udf::{CustomResolverRequestPayload, UdfResponse};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[test]
fn aliases() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_petstore_engine(petstore_schema(mock_server.address())).await;

        mock_doggo(&mock_server, 123, "Immediate Doggo").await;
        mock_doggo(&mock_server, 456, "Deferred Doggo").await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query {
                        petstore {
                            goodDoggo: pet(petId: 123) {
                                id
                                name
                            }
                            veryGoodDoggo: pet(petId: 456) {
                                id
                                name
                            }
                        }
                    }
                ",
                )
                .await
                .into_value(),
            @r###"
        {
          "data": {
            "petstore": {
              "goodDoggo": {
                "id": 123,
                "name": "Immediate Doggo"
              },
              "veryGoodDoggo": {
                "id": 456,
                "name": "Deferred Doggo"
              }
            }
          }
        }
        "###
        );
    });
}

#[test]
fn test_nullable_list_validation() {
    runtime().block_on(async {
        let schema = r#"
            extend type Query {
                list: [[[Nested!]!]] @resolver(name: "list")
            }

            type Nested {
                name: String @resolver(name: "name")
            }
        "#;
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver(
                        "list",
                        UdfResponse::Success(json!([null, [["hello"]], [["world"], null]])),
                    )
                    .resolver("name", UdfResponse::Success(json!("Jim"))),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine.execute("query { list { name } }").await.into_value(),
            @r###"
        {
          "data": {
            "list": [
              null,
              [
                [
                  {
                    "name": "Jim"
                  }
                ]
              ],
              null
            ]
          },
          "errors": [
            {
              "message": "An error occurred while fetching `list`, a non-nullable value was expected but no value was found.",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "list",
                2,
                1
              ]
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn test_nullable_list_item_validation() {
    runtime().block_on(async {
        let schema = r#"
            extend type Query {
                list: [[[Nested!]!]] @resolver(name: "list")
            }

            type Nested {
                name: String! @resolver(name: "name")
            }
        "#;
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver("list", UdfResponse::Success(json!([[["hello", ""]], [["world"]]])))
                    .resolver("name", |payload: CustomResolverRequestPayload| {
                        if payload.parent == Some(json!("world")) {
                            Ok(UdfResponse::Success(json!(null)))
                        } else {
                            Ok(UdfResponse::Success(json!("Jim")))
                        }
                    }),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine.execute("query { list { name } }").await.into_value(),
            @r###"
        {
          "data": {
            "list": [
              [
                [
                  {
                    "name": "Jim"
                  },
                  {
                    "name": "Jim"
                  }
                ]
              ],
              null
            ]
          },
          "errors": [
            {
              "message": "An error happened while fetching `name`, expected a non null value but found a null",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "path": [
                "list",
                1,
                0,
                0,
                "name"
              ]
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn test_nested_lists() {
    runtime().block_on(async {
        let schema = r#"
            extend type Query {
                list: [[[Nested]]!] @resolver(name: "list")
            }

            type Nested {
                name: String @resolver(name: "name")
            }
        "#;
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver("list", UdfResponse::Success(json!([[["world"]], [["hello"]]])))
                    .resolver("name", UdfResponse::Success(json!("Jim"))),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine.execute("query { list { name } }").await.into_value(),
            @r###"
        {
          "data": {
            "list": [
              [
                [
                  {
                    "name": "Jim"
                  }
                ]
              ],
              [
                [
                  {
                    "name": "Jim"
                  }
                ]
              ]
            ]
          }
        }
        "###
        );
    });
}

#[test]
fn nested_fragment_resolution() {
    // We had a bug where fragments (particularly nested ones)
    // that selected the same fields ended up overwriting each other.
    // This covers that case.
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(FakeGithubSchema).await;
        let schema = indoc::formatdoc! {
            r#"
              extend schema
                @graphql(
                    name: "gothub",
                    namespace: false,
                    url: "http://127.0.0.1:{}",
                )
            "#,
            graphql_mock.port()
        };

        let engine = EngineBuilder::new(schema).build().await;

        const QUERY: &str = indoc::indoc! {
            r#"
                query {
                    pullRequestsAndIssues(filter: {search: ""}) {
                        ... on PullRequest {
                            checks
                            author {
                                # This nested fragment spread should not clash
                                # with the one used in the inline fragment below
                                ...AuthorFragmentOne
                            }
                        }

                        # This second fragment should not overwrite the checks
                        # field from the previous fragment
                        ... on PullRequest {
                            author {
                                # This nested fragment spread should not clash
                                # with the one used in the inline fragment above
                                ...AuthorFragmentTwo
                            }
                        }
                    }
                }

                fragment AuthorFragmentOne on User {
                    name
                }

                fragment AuthorFragmentTwo on User {
                    email
                }
            "#
        };

        insta::assert_json_snapshot!(engine.execute(QUERY).variables(json!({"id": "1"})).await.into_value(), @r###"
        {
          "data": {
            "pullRequestsAndIssues": [
              {
                "checks": [
                  "Success!"
                ],
                "author": {
                  "name": "Jim",
                  "email": "jim@example.com"
                }
              },
              {
                "checks": [
                  "Success!"
                ],
                "author": {}
              },
              {}
            ]
          }
        }
        "###);
    });
}

async fn build_petstore_engine(schema: String) -> Engine {
    EngineBuilder::new(schema)
        .with_openapi_schema(
            "http://example.com/petstore.json",
            include_str!("../openapi/petstore.json"),
        )
        .build()
        .await
}

fn petstore_schema(address: &SocketAddr) -> String {
    format!(
        r#"
          extend schema
          @openapi(
            name: "petstore",
            url: "http://{address}",
            schema: "http://example.com/petstore.json",
          )
        "#
    )
}

async fn mock_doggo(mock_server: &MockServer, id: u32, name: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/pet/{id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(doggo(id, name)))
        .mount(mock_server)
        .await;
}

fn doggo(id: u32, name: &str) -> serde_json::Value {
    json!({
        "id": id,
        "name": name,
        "category": {
            "id": 1,
            "name": "Dogs"
        },
        "photoUrls": [
            "string"
        ],
        "tags": [
            {
            "id": 0,
            "name": "string"
            }
        ],
        "status": "available"
    })
}
