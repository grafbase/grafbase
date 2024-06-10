use integration_tests::{runtime, udfs::RustUdfs, EngineBuilder, ResponseExt};
use runtime::udf::{CustomResolverRequestPayload, UdfResponse};
use serde_json::json;

use crate::subgraph::todo_engine;

#[test]
fn unknown_entity() {
    runtime().block_on(async {
        let engine = todo_engine([]).await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query($repr: _Any!) {
                        _entities(representations: [$repr]) {
                            __typename
                        }
                    }
                ",
                )
                .variables(json!({"repr": {
                    "__typename": "SomeUnknownType",
                    "id": "123"
                }}))
                .await
                .into_value(),
                @r###"
        {
          "data": {
            "_entities": [
              null
            ]
          },
          "errors": [
            {
              "message": "Unknown __typename in representation: SomeUnknownType",
              "locations": [
                {
                  "line": 3,
                  "column": 25
                }
              ],
              "path": [
                "_entities",
                0
              ]
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn unknown_key() {
    runtime().block_on(async {
        let engine = todo_engine([]).await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query($repr: _Any!) {
                        _entities(representations: [$repr]) {
                            __typename
                            ... on Todo {
                                title
                            }
                        }
                    }
                ",
                )
                .variables(json!({"repr": {
                    "__typename": "Todo",
                    "some_unknown_field": "123"
                }}))
                .await
                .into_value(),
                @r###"
        {
          "data": {
            "_entities": [
              null
            ]
          },
          "errors": [
            {
              "message": "Could not find a matching key for the given representation",
              "locations": [
                {
                  "line": 3,
                  "column": 25
                }
              ],
              "path": [
                "_entities",
                0
              ]
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn partial_failures() {
    runtime().block_on(async {
        let engine = todo_engine([serde_json::json!({
            "id": "todo_1",
            "title": "Test Federation",
        })])
        .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query($reprs: [_Any!]!) {
                        _entities(representations: $reprs) {
                            __typename
                            ... on Todo {
                                title
                            }
                        }
                    }
                ",
                )
                .variables(json!({"reprs": [
                    { "__typename": "Todo", "id": "todo_1" },
                    { "__typename": "SomeUnknownType", "id": "todo_1" }
                ]}))
                .await
                .into_value(),
                @r###"
        {
          "data": {
            "_entities": [
              {
                "__typename": "Todo",
                "title": "Test Federation"
              },
              null
            ]
          },
          "errors": [
            {
              "message": "Unknown __typename in representation: SomeUnknownType",
              "locations": [
                {
                  "line": 3,
                  "column": 25
                }
              ],
              "path": [
                "_entities",
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
fn totally_malformed_representation() {
    runtime().block_on(async {
        const SCHEMA: &str = r#"
            extend schema @federation(version: "2.3")

            extend type Query {
                todo(id: ID!): Todo @resolver(name: "todo")
            }

            type Todo @key(fields: "id", select: "todo(id: $id)") {
                id: ID!
                title: String!
            }
        "#;

        let engine = EngineBuilder::new(SCHEMA)
            .with_custom_resolvers(
                RustUdfs::new().resolver("todo", move |_payload: CustomResolverRequestPayload| {
                    Ok(UdfResponse::Success(json!(null)))
                }),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query($reprs: [_Any!]!) {
                        _entities(representations: $reprs) {
                            __typename
                            ... on Todo {
                                title
                            }
                        }
                    }
                ",
                )
                .variables(json!({"reprs": [
                    "this is a string when it should be an object"
                ]}))
                .await
                .into_value(),
                @r###"
        {
          "errors": [
            {
              "message": "Invalid value for argument \"representations.0\", expected type \"_Any\"",
              "locations": [
                {
                  "line": 3,
                  "column": 35
                }
              ]
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn representation_missing_typename() {
    runtime().block_on(async {
        const SCHEMA: &str = r#"
            extend schema @federation(version: "2.3")

            extend type Query {
                todo(id: ID!): Todo @resolver(name: "todo")
            }

            type Todo @key(fields: "id", select: "todo(id: $id)") {
                id: ID!
                title: String!
            }
        "#;

        let engine = EngineBuilder::new(SCHEMA)
            .with_custom_resolvers(
                RustUdfs::new().resolver("todo", move |_payload: CustomResolverRequestPayload| {
                    Ok(UdfResponse::Success(json!(null)))
                }),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query($reprs: [_Any!]!) {
                        _entities(representations: $reprs) {
                            __typename
                            ... on Todo {
                                title
                            }
                        }
                    }
                ",
                )
                .variables(json!({"reprs": [
                    { "blah": "this is missing __typename" },
                ]}))
                .await
                .into_value(),
                @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Could not deserialize _Any: missing field `__typename`",
              "locations": [
                {
                  "line": 3,
                  "column": 25
                }
              ],
              "path": [
                "_entities"
              ]
            }
          ]
        }
        "###
        );
    });
}
