#![allow(unused_crate_dependencies)]
mod utils;

use crate::utils::consts::{
    RESERVED_DATES_CREATE_TODO, RESERVED_DATES_CREATE_TODO_LIST, RESERVED_DATES_NESTED_CREATION, RESERVED_DATES_SCHEMA,
};
use backend::project::GraphType;
use chrono::{DateTime, Duration, Utc};
use serde_json::{json, Value};
use utils::environment::Environment;

#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reserved_dates() {
    // TODO: Create simpler client setup (one-line)
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(RESERVED_DATES_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300).await;

    let response = client.gql::<Value>(RESERVED_DATES_NESTED_CREATION).send().await;

    let user: Value = dot_get!(response, "data.userCreate.user");
    assert_eq!(dot_get!(user, "name", String), "John");
    assert_eq!(dot_get!(user, "email", String), "john@example.org");
    assert!(!dot_get!(user, "id", String).is_empty());
    let created_at = dot_get!(user, "createdAt", String);
    assert!(created_at
        .parse::<DateTime<Utc>>()
        .expect("Should have a valid datetime")
        .gt(&Utc::now().checked_sub_signed(Duration::hours(1)).unwrap()));
    assert_eq!(dot_get!(user, "updatedAt", String), created_at);

    let post: Value = dot_get!(user, "posts.edges.0.node");
    assert_eq!(dot_get!(post, "title", String), "10 ways to pet your dog!");
    assert_eq!(dot_get!(post, "content", String), "Dogs are the best.");
    // Ensuring we do generate every reserved field as expected.
    assert!(!dot_get!(post, "id", String).is_empty());
    assert_eq!(dot_get!(post, "createdAt", String), created_at);
    assert_eq!(dot_get!(post, "updatedAt", String), created_at);

    let comment: Value = dot_get!(post, "comments.edges.0.node");
    assert_eq!(dot_get!(comment, "content", String), "Awesome!");
    assert!(!dot_get!(comment, "id", String).is_empty());
    assert_eq!(dot_get!(comment, "createdAt", String), created_at);
    assert_eq!(dot_get!(comment, "updatedAt", String), created_at);

    let nested_user: Value = dot_get!(comment, "user");
    assert_eq!(dot_get!(nested_user, "name", String), "Alice");
    assert_eq!(dot_get!(nested_user, "email", String), "alice@example.org");
    assert!(!dot_get!(nested_user, "id", String).is_empty());
    assert_ne!(dot_get!(nested_user, "id", String), dot_get!(user, "id", String));
    assert_eq!(dot_get!(nested_user, "createdAt", String), created_at);
    assert_eq!(dot_get!(nested_user, "updatedAt", String), created_at);

    let response = client
        .gql::<Value>(RESERVED_DATES_CREATE_TODO)
        .variables(json!({
                "title": "Champion"
        }))
        .send()
        .await;
    let todo: Value = dot_get!(response, "data.todoCreate.todo");
    let todo_created_at = dot_get!(todo, "createdAt", String);
    assert!(todo_created_at
        .parse::<DateTime<Utc>>()
        .expect("Should have a valid datetime")
        .gt(&Utc::now().checked_sub_signed(Duration::hours(1)).unwrap()));
    assert_eq!(dot_get!(todo, "updatedAt", String), todo_created_at);

    let response = client
        .gql::<Value>(RESERVED_DATES_CREATE_TODO_LIST)
        .variables(json!({
            "title": "Champion List",
            "todoId": dot_get!(todo, "id", String)
        }))
        .send()
        .await;
    let todo_list: Value = dot_get!(response, "data.todoListCreate.todoList");
    let todo_list_created_at = dot_get!(todo_list, "createdAt", String);
    assert!(todo_list_created_at
        .parse::<DateTime<Utc>>()
        .expect("Should have a valid datetime")
        .gt(&Utc::now().checked_sub_signed(Duration::hours(1)).unwrap()));
    assert_eq!(dot_get!(todo_list, "updatedAt", String), todo_list_created_at);
    assert_ne!(todo_created_at, todo_list_created_at);

    let nested_todo: Value = dot_get!(todo_list, "todos.edges.0.node");
    assert_eq!(dot_get!(nested_todo, "createdAt", String), todo_created_at);
    assert_eq!(dot_get!(nested_todo, "updatedAt", String), todo_created_at);
}
