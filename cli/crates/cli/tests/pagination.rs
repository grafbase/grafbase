#![allow(clippy::too_many_lines)]

mod utils;

use serde_json::{json, Value};
use utils::consts::{
    PAGINATION_MUTATION, PAGINATION_PAGINATE_TODOS, PAGINATION_PAGINATE_TODO_LISTS, PAGINATION_SCHEMA,
};
use utils::environment::Environment;

#[test]

fn dev() {
    let mut env = Environment::init(4010);

    env.grafbase_init();

    env.write_schema(PAGINATION_SCHEMA);

    env.grafbase_dev();

    let client = env.create_client();

    client.poll_endpoint(30, 300);

    client.gql::<Value>(
        json!({
            "query": PAGINATION_MUTATION,
            "variables": { "title": "1" }
        })
        .to_string(),
    );

    let response = client.gql::<Value>(
        json!({
            "query": PAGINATION_PAGINATE_TODOS,
            "variables": {
                "last": 1
            }
        })
        .to_string(),
    );

    let last_todo: Value = dot_get!(response, "data.todoCollection.edges.0.node");

    let last_todo_id: String = dot_get!(last_todo, "id");
    let last_todo_title: String = dot_get!(last_todo, "title");

    assert!(last_todo_id.starts_with("todo_"));
    assert_eq!(last_todo_title, "1");

    let has_next_page: bool = dot_get!(response, "data.todoCollection.pageInfo.hasNextPage");
    let has_previous_page: bool = dot_get!(response, "data.todoCollection.pageInfo.hasPreviousPage");

    assert!(!has_next_page);
    assert!(has_previous_page);

    let response = client.gql::<Value>(
        json!({
            "query": PAGINATION_PAGINATE_TODOS,
            "variables": {
                "last": 10,
                "before": last_todo_id
            }
        })
        .to_string(),
    );

    let next_todo: Value = dot_get!(response, "data.todoCollection.edges.0.node");

    let next_todo_id: String = dot_get!(next_todo, "id");
    let next_todo_title: String = dot_get!(next_todo, "title");

    assert!(next_todo_id != last_todo_id);
    assert_eq!(next_todo_title, "2");

    let has_next_page: bool = dot_get!(response, "data.todoCollection.pageInfo.hasNextPage");
    let has_previous_page: bool = dot_get!(response, "data.todoCollection.pageInfo.hasPreviousPage");

    assert!(!has_next_page);
    assert!(!has_previous_page);

    let response = client.gql::<Value>(
        json!({
            "query": PAGINATION_PAGINATE_TODOS,
            "variables": {
                "first": 1
            }
        })
        .to_string(),
    );

    let first_todo: Value = dot_get!(response, "data.todoCollection.edges.0.node");

    let first_todo_id: String = dot_get!(first_todo, "id");
    let first_todo_title: String = dot_get!(first_todo, "title");

    assert!(first_todo_id.starts_with("todo_"));
    assert_eq!(first_todo_title, "3");

    let response = client.gql::<Value>(
        json!({
            "query": PAGINATION_PAGINATE_TODOS,
            "variables": {
                "first": 1,
                "after": first_todo_id
            }
        })
        .to_string(),
    );

    let next_todo: Value = dot_get!(response, "data.todoCollection.edges.0.node");

    let has_next_page: bool = dot_get!(response, "data.todoCollection.pageInfo.hasNextPage");
    let has_previous_page: bool = dot_get!(response, "data.todoCollection.pageInfo.hasPreviousPage");

    assert!(has_next_page);
    assert!(!has_previous_page);

    let next_todo_id: String = dot_get!(next_todo, "id");
    let next_todo_title: String = dot_get!(next_todo, "title");

    assert!(next_todo_id != first_todo_id);
    assert_eq!(next_todo_title, "2");

    let response = client.gql::<Value>(
        json!({
            "query": PAGINATION_PAGINATE_TODOS,
            "variables": {
                "first": 10,
                "after": first_todo_id
            }
        })
        .to_string(),
    );

    let next_todo: Value = dot_get!(response, "data.todoCollection.edges.0.node");

    let next_todo_id: String = dot_get!(next_todo, "id");
    let next_todo_title: String = dot_get!(next_todo, "title");

    assert!(next_todo_id != first_todo_id);
    assert_eq!(next_todo_title, "2");

    let has_next_page: bool = dot_get!(response, "data.todoCollection.pageInfo.hasNextPage");
    let has_previous_page: bool = dot_get!(response, "data.todoCollection.pageInfo.hasPreviousPage");

    assert!(!has_next_page);
    assert!(!has_previous_page);

    client.gql::<Value>(
        json!({
            "query": PAGINATION_MUTATION,
            "variables": { "title": "2" }
        })
        .to_string(),
    );

    client.gql::<Value>(
        json!({
            "query": PAGINATION_MUTATION,
            "variables": { "title": "3" }
        })
        .to_string(),
    );

    let response = client.gql::<Value>(
        json!({
            "query": PAGINATION_PAGINATE_TODO_LISTS,
            "variables": {
                "first": 1
            }
        })
        .to_string(),
    );

    let last_todo_list: Value = dot_get!(response, "data.todoListCollection.edges.0.node");

    let last_todo_list_id: String = dot_get!(last_todo_list, "id");
    let last_todo_list_title: String = dot_get!(last_todo_list, "title");

    assert!(last_todo_list_id.starts_with("todolist_"));
    assert_eq!(last_todo_list_title, "3");

    let has_next_page: bool = dot_get!(response, "data.todoListCollection.pageInfo.hasNextPage");
    let has_previous_page: bool = dot_get!(response, "data.todoListCollection.pageInfo.hasPreviousPage");

    assert!(has_next_page);
    assert!(!has_previous_page);

    let response = client.gql::<Value>(
        json!({
            "query": PAGINATION_PAGINATE_TODO_LISTS,
            "variables": {
                "first": 10,
                "after": last_todo_list_id
            }
        })
        .to_string(),
    );

    let next_todo_list: Value = dot_get!(response, "data.todoListCollection.edges.0.node");

    let next_todo_list_id: String = dot_get!(next_todo_list, "id");
    let next_todo_list_title: String = dot_get!(next_todo_list, "title");

    assert!(next_todo_list_id != last_todo_list_id);
    assert_eq!(next_todo_list_title, "2");

    let response = client.gql::<Value>(
        json!({
            "query": PAGINATION_PAGINATE_TODO_LISTS,
            "variables": {
                "first": 10,
                "after": last_todo_list_id
            }
        })
        .to_string(),
    );

    let next_todo_list: Value = dot_get!(response, "data.todoListCollection.edges.0.node");

    let next_todo_list_id: String = dot_get!(next_todo_list, "id");
    let next_todo_list_title: String = dot_get!(next_todo_list, "title");

    assert!(next_todo_list_id != last_todo_list_id);
    assert_eq!(next_todo_list_title, "2");

    let has_next_page: bool = dot_get!(response, "data.todoListCollection.pageInfo.hasNextPage");
    let has_previous_page: bool = dot_get!(response, "data.todoListCollection.pageInfo.hasPreviousPage");

    assert!(!has_next_page);
    assert!(!has_previous_page);

    //3

    let response = client.gql::<Value>(
        json!({
            "query": PAGINATION_PAGINATE_TODO_LISTS,
            "variables": {
                "last": 1
            }
        })
        .to_string(),
    );

    let last_todo_list: Value = dot_get!(response, "data.todoListCollection.edges.0.node");

    let last_todo_list_id: String = dot_get!(last_todo_list, "id");
    let last_todo_list_title: String = dot_get!(last_todo_list, "title");

    assert!(last_todo_list_id.starts_with("todolist_"));
    assert_eq!(last_todo_list_title, "1");

    let has_next_page: bool = dot_get!(response, "data.todoListCollection.pageInfo.hasNextPage");
    let has_previous_page: bool = dot_get!(response, "data.todoListCollection.pageInfo.hasPreviousPage");

    assert!(!has_next_page);
    assert!(has_previous_page);

    let response = client.gql::<Value>(
        json!({
            "query": PAGINATION_PAGINATE_TODO_LISTS,
            "variables": {
                "last": 1,
                "before": last_todo_list_id
            }
        })
        .to_string(),
    );

    let next_todo_list: Value = dot_get!(response, "data.todoListCollection.edges.0.node");

    let next_todo_list_id: String = dot_get!(next_todo_list, "id");
    let next_todo_list_title: String = dot_get!(next_todo_list, "title");

    assert!(next_todo_list_id != last_todo_list_id);
    assert_eq!(next_todo_list_title, "2");

    let response = client.gql::<Value>(
        json!({
            "query": PAGINATION_PAGINATE_TODO_LISTS,
            "variables": {
                "last": 10,
                "before": last_todo_list_id
            }
        })
        .to_string(),
    );

    let next_todo_list: Value = dot_get!(response, "data.todoListCollection.edges.0.node");

    let next_todo_list_id: String = dot_get!(next_todo_list, "id");
    let next_todo_list_title: String = dot_get!(next_todo_list, "title");

    assert!(next_todo_list_id != last_todo_list_id);
    assert_eq!(next_todo_list_title, "2");

    let has_next_page: bool = dot_get!(response, "data.todoListCollection.pageInfo.hasNextPage");
    let has_previous_page: bool = dot_get!(response, "data.todoListCollection.pageInfo.hasPreviousPage");

    assert!(!has_next_page);
    assert!(!has_previous_page);
}
