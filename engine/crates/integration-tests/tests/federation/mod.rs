mod errors;
mod introspection;

use integration_tests::{runtime, Engine, EngineBuilder, ResponseExt};
use serde_json::{json, Value};

const TODO_SCHEMA: &str = r#"
    extend schema @federation(version: "2.3")

    type Todo @model {
        id: ID!
        title: String!
    }
"#;

#[test]
fn federation_smoke_test() {
    runtime().block_on(async {
        let engine = EngineBuilder::new(TODO_SCHEMA).with_local_dynamo().build().await;

        let todo_id = engine.create_todo("Test Federation").await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r#"
                    query($repr: _Any!) {
                        _entities(representations: [$repr]) {
                            __typename
                            ... on Todo {
                                title
                            }
                        }
                    }
                "#,
                )
                .variables(json!({"repr": {
                    "__typename": "Todo",
                    "id": todo_id
                }}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "_entities": [
            {
              "__typename": "Todo",
              "title": "Test Federation"
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn test_getting_multiple_reprs() {
    runtime().block_on(async {
        let engine = EngineBuilder::new(TODO_SCHEMA).with_local_dynamo().build().await;

        let todo_id_one = engine.create_todo("Test Federation").await;
        let todo_id_two = engine.create_todo("Release Federation").await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r#"
                    query($reprs: [_Any!]!) {
                        _entities(representations: $reprs) {
                            __typename
                            ... on Todo {
                                title
                            }
                        }
                    }
                "#,
                )
                .variables(json!({"reprs": [
                    { "__typename": "Todo", "id": todo_id_one },
                    { "__typename": "Todo", "id": todo_id_two },
                ]}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "_entities": [
            {
              "__typename": "Todo",
              "title": "Test Federation"
            },
            {
              "__typename": "Todo",
              "title": "Release Federation"
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn test_missing_item() {
    runtime().block_on(async {
        let engine = EngineBuilder::new(TODO_SCHEMA).with_local_dynamo().build().await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r#"
                    query($repr: _Any!) {
                        _entities(representations: [$repr]) {
                            __typename
                            ... on Todo {
                                title
                            }
                        }
                    }
                "#,
                )
                .variables(json!({"repr": {
                    "__typename": "Todo",
                    "id": "123"
                }}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "_entities": [
            null
          ]
        }
        "###
        );
    });
}

#[async_trait::async_trait]
trait TodoEngineExt {
    /// Creates a todo with this engine, returns a string
    async fn create_todo(&self, title: &str) -> String;
}

#[async_trait::async_trait]
impl TodoEngineExt for Engine {
    async fn create_todo(&self, title: &str) -> String {
        self.execute(
            r#"
                mutation($title: String!) {
                    todoCreate(input: {title: $title}) {
                        todo {
                            id
                        }
                    }
                }
            "#,
        )
        .variables(json!({"title": title}))
        .await
        .into_data::<Value>()["todoCreate"]["todo"]["id"]
            .as_str()
            .unwrap()
            .to_string()
    }
}
