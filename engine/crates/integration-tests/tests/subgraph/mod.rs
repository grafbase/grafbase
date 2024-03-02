//! Tests of Grafbase acting as a federation subgraph

mod errors;
mod introspection;
mod keys;
mod openapi;
mod requires;

use integration_tests::{runtime, udfs::RustUdfs, Engine, EngineBuilder, ResponseExt};
use runtime::udf::{CustomResolverRequestPayload, UdfError, UdfResponse};
use serde_json::{json, Value};

const TOOD_SCHEMA: &str = r#"
    extend schema @federation(version: "2.3")

    extend type Query {
        todo(id: ID!): Todo @resolver(name: "todo")
    }

    type Todo @key(fields: "id", select: "todo(id: $id)") {
        id: ID!
        title: String!
    }
"#;

fn resolver_returning_items_by_id(
    items: impl IntoIterator<Item = serde_json::Value>,
) -> Box<dyn Fn(CustomResolverRequestPayload) -> Result<UdfResponse, UdfError> + Send + Sync> {
    let items: std::collections::HashMap<_, _> = items
        .into_iter()
        .map(|item| {
            let id = item["id"].as_str().expect("ID must be a string");
            (id.to_owned(), item)
        })
        .collect();
    Box::new(move |payload| {
        Ok(UdfResponse::Success(
            payload.arguments["id"]
                .as_str()
                .and_then(|id| items.get(id).cloned())
                .unwrap_or(serde_json::Value::Null),
        ))
    })
}

async fn todo_engine(items: impl IntoIterator<Item = serde_json::Value>) -> Engine {
    EngineBuilder::new(TOOD_SCHEMA)
        .with_custom_resolvers(RustUdfs::new().resolver("todo", resolver_returning_items_by_id(items)))
        .build()
        .await
}

#[test]
fn federation_smoke_test() {
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
                    "id": "todo_1"
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
        let engine = todo_engine([
            serde_json::json!({
                "id": "todo_1",
                "title": "Test Federation",
            }),
            serde_json::json!({
                "id": "todo_2",
                "title": "Release Federation",
            }),
        ])
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
                    { "__typename": "Todo", "id": "todo_2" },
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
        let engine: Engine = todo_engine([]).await;

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

            extend type Query {
                todo(id: ID!): Todo @resolver(name: "todo")
            }
    
            type Todo @key(fields: "id", select: "todo(id: $id)") {
                id: ID!
                title: String!
                todoList: TodoList @resolver(name: "todoListRepresentation")
                todoListId: ID
            }

            type TodoList @key(fields: "id", resolvable: false) {
                id: ID!
            }
        "#;

        let todo_without_list_id = "todo_1";
        let todo_without_list = serde_json::json!({
            "id": todo_without_list_id,
            "title": "Test a todo with no list",
            "todoListId": None::<String>,
        });
        let todolist_id = "123";
        let todo_with_list_id = "todo_2";
        let todo_with_list = serde_json::json!({
            "id": todo_without_list_id,
            "title": "Test a todo with a list",
            "todoListId": todolist_id,
        });

        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver("todo", move |payload: CustomResolverRequestPayload| {
                        Ok(UdfResponse::Success(match payload.arguments["id"].as_str() {
                            Some(id) => {
                                if id == todo_without_list_id {
                                    todo_without_list.clone()
                                } else if id == todo_with_list_id {
                                    todo_with_list.clone()
                                } else {
                                    json!(null)
                                }
                            }
                            _ => json!(null),
                        }))
                    })
                    .resolver("todoListRepresentation", |payload: CustomResolverRequestPayload| {
                        Ok(if let Some(id) = payload.parent.unwrap()["todoListId"].as_str() {
                            UdfResponse::Success(json!({ "id": id }))
                        } else {
                            UdfResponse::Success(json!(null))
                        })
                    }),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query($withoutListId: ID!, $withListId: ID!) {
                        withoutList: todo(id: $withoutListId) {
                            title
                            # Annoying having to query the ID for this
                            # we should do https://linear.app/grafbase/issue/GB-4014 sometime
                            todoListId
                            todoList {
                                id
                                __typename
                            }
                        }
                        withList: todo(id: $withListId) {
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
                ",
                )
                .variables(json!({"withoutListId": todo_without_list_id, "withListId": todo_with_list_id}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "withoutList": {
            "title": "Test a todo with no list",
            "todoListId": null,
            "todoList": null
          },
          "withList": {
            "title": "Test a todo with a list",
            "todoListId": "123",
            "todoList": {
              "id": "123",
              "__typename": "TodoList"
            }
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
            .with_custom_resolvers(
                RustUdfs::new().resolver("todoListName", |payload: CustomResolverRequestPayload| {
                    let parent = payload.parent.unwrap();
                    let id = parent["id"].as_str().unwrap();
                    Ok(UdfResponse::Success(json!(format!("A List With ID {id}"))))
                }),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query($repr: _Any!) {
                        _entities(representations: [$repr]) {
                            __typename
                            ... on TodoList {
                                id
                                name
                            }
                        }
                    }
                ",
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

#[test]
fn test_key_with_select() {
    // Tests that keys with a select parameter correctly resolve using the
    // join resolver

    runtime().block_on(async {
        let schema = r#"
            extend schema @federation(version: "2.3")

            extend type Query {
                todoList(id: ID!): TodoList! @resolver(name: "todoList")
            }

            type TodoList @key(fields: "id", select: "todoList(id: $id)") {
                id: ID!
                name: String!
            }
        "#;

        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new().resolver("todoList", |payload: CustomResolverRequestPayload| {
                    let id = payload.arguments["id"].as_str().unwrap();
                    Ok(UdfResponse::Success(
                        json!({"id": id, "name": format!("A list With Id {id}")}),
                    ))
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
                            ... on TodoList {
                                id
                                name
                            }
                        }
                    }
                ",
                )
                .variables(json!({"reprs": [
                    { "__typename": "TodoList", "id": "123" },
                    { "__typename": "TodoList", "id": "456" },
                ]}))
                .await
                .into_data::<Value>(),
                @r###"
        {
          "_entities": [
            {
              "__typename": "TodoList",
              "id": "123",
              "name": "A list With Id 123"
            },
            {
              "__typename": "TodoList",
              "id": "456",
              "name": "A list With Id 456"
            }
          ]
        }
        "###
        );
    });
}
