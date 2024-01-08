use integration_tests::{runtime, EngineBuilder, ResponseExt};
use serde_json::json;

use super::{TodoEngineExt, TODO_SCHEMA};

#[test]
fn unknown_entity() {
    runtime().block_on(async {
        let engine = EngineBuilder::new(TODO_SCHEMA).build().await;

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
        let engine = EngineBuilder::new(TODO_SCHEMA).build().await;

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
        let engine = EngineBuilder::new(TODO_SCHEMA).build().await;

        let todo_id = engine.create_todo("Test Federation").await;

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
                    { "__typename": "Todo", "id": todo_id },
                    { "__typename": "SomeUnknownType", "id": todo_id }
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
        let engine = EngineBuilder::new(TODO_SCHEMA).build().await;

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
          "data": null,
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
        let engine = EngineBuilder::new(TODO_SCHEMA).build().await;

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
