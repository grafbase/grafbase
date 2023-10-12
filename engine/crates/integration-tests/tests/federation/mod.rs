mod errors;
mod introspection;

use integration_tests::{runtime, udfs::RustUdfs, Engine, EngineBuilder, ResponseExt};
use runtime::udf::{CustomResolverRequestPayload, CustomResolverResponse};
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

#[test]
fn test_returning_unresolvable_representations() {
    // Tests that we can return an unresolvable representation for another subgraphs
    // entity as part of a normal query.
    // This isn't really a test of specific code, just makes sure the pattern works
    // as expected

    runtime().block_on(async {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type Todo @model {
                id: ID!
                title: String!
                todoList: TodoList @resolver(name: "todoListRepresentation")
                todoListId: ID
            }

            type TodoList @key(fields: "id", resolvable: false) {
                id: ID!
            }
        "#;

        let engine = EngineBuilder::new(schema)
            .with_local_dynamo()
            .with_custom_resolvers(RustUdfs::new().resolver(
                "todoListRepresentation",
                |payload: CustomResolverRequestPayload| {
                    Ok(if let Some(id) = payload.parent.unwrap()["todoListId"].as_str() {
                        CustomResolverResponse::Success(json!({ "id": id }))
                    } else {
                        CustomResolverResponse::Success(json!(null))
                    })
                },
            ))
            .build()
            .await;

        let todo_id = engine.create_todo("Test a todo with no list").await;
        let todo_with_list_id = engine.create_todo_with_list("Test a todo with a list", "123").await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r#"
                    query($withoutListId: ID!, $withListId: ID!) {
                        withoutList: todo(by: {id: $withoutListId}) {
                            title
                            # Annoying having to query the ID for this
                            # we should do https://linear.app/grafbase/issue/GB-4014 sometime
                            todoListId
                            todoList {
                                id
                                __typename
                            }
                        }
                        withList: todo(by: {id: $withListId}) {
                            title
                            # Annoying having to query the ID for this
                            # we should do https://linear.app/grafbase/issue/GB-4014 sometime
                            todoListId
                            todoList {
                                id
                                __typename
                            }
                        }
                    }
                "#,
                )
                .variables(json!({"withoutListId": todo_id, "withListId": todo_with_list_id}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "withList": {
            "title": "Test a todo with a list",
            "todoList": {
              "__typename": "TodoList",
              "id": "123"
            },
            "todoListId": "123"
          },
          "withoutList": {
            "title": "Test a todo with no list",
            "todoList": null,
            "todoListId": null
          }
        }
        "###
        );
    });
}

#[test]
fn test_contributing_fields_via_default_resolver() {
    // Tests that we can take in a representation and add fields to it via custom
    // resolvers.

    runtime().block_on(async {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type TodoList @key(fields: "id") {
                id: ID!
                name: String! @resolver(name: "todoListName")
            }
        "#;

        let engine = EngineBuilder::new(schema)
            .with_local_dynamo()
            .with_custom_resolvers(
                RustUdfs::new().resolver("todoListName", |payload: CustomResolverRequestPayload| {
                    let parent = payload.parent.unwrap();
                    let id = parent["id"].as_str().unwrap();
                    Ok(CustomResolverResponse::Success(json!(format!("A List With ID {id}"))))
                }),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r#"
                    query($repr: _Any!) {
                        _entities(representations: [$repr]) {
                            __typename
                            ... on TodoList {
                                id
                                name
                            }
                        }
                    }
                "#,
                )
                .variables(json!({"repr": {
                    "__typename": "TodoList",
                    "id": "123"
                }}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "_entities": [
            {
              "__typename": "TodoList",
              "id": "123",
              "name": "A List With ID 123"
            }
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

    /// Creates a todo with this engine, returns a string
    /// Note that this requires a different schema from the above
    async fn create_todo_with_list(&self, title: &str, list_id: &str) -> String;
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

    async fn create_todo_with_list(&self, title: &str, list_id: &str) -> String {
        self.execute(
            r#"
                mutation($title: String!, $listId: ID) {
                    todoCreate(input: {title: $title, todoListId: $listId}) {
                        todo {
                            id
                        }
                    }
                }
            "#,
        )
        .variables(json!({"title": title, "listId": list_id}))
        .await
        .into_data::<Value>()["todoCreate"]["todo"]["id"]
            .as_str()
            .unwrap()
            .to_string()
    }
}
