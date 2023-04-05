mod utils;

use serde_json::{json, Value};
use utils::consts::{DEFAULT_CREATE, DEFAULT_QUERY, DEFAULT_SCHEMA, DEFAULT_UPDATE};
use utils::environment::Environment;

#[test]
fn dev() {
    let mut env = Environment::init();
    env.grafbase_init();
    env.write_schema(DEFAULT_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    //
    // CREATE
    //
    let response = client.gql::<Value>(DEFAULT_CREATE).send();
    let todo_list: Value = dot_get!(response, "data.todoListCreate.todoList");
    let todo_list_id: String = dot_get!(todo_list, "id");
    assert!(!todo_list_id.is_empty());
    assert_eq!(dot_get!(todo_list, "title", String), "My todo list");

    let todos: Vec<Value> = dot_get!(todo_list, "todos.edges", Value)
        .as_array()
        .map(|array| {
            array
                .iter()
                .map(|element| dot_get!(element, "node", Value))
                .collect::<Vec<_>>()
        })
        .unwrap();
    assert_eq!(todos.len(), 2);
    assert_eq!(dot_get!(todos[0], "title", String), "My first todo!");
    assert!(dot_get!(todos[0], "complete", bool));
    assert_eq!(dot_get!(todos[1], "title", String), "My second todo!");
    assert!(!dot_get!(todos[1], "complete", bool));
    assert_ne!(dot_get!(todos[0], "id", String), dot_get!(todos[1], "id", String));

    //
    // QUERY
    //
    let response = client.gql::<Value>(DEFAULT_QUERY).send();
    let edges: Value = dot_get!(response, "data.todoListCollection.edges");
    assert_eq!(edges.as_array().map(Vec::len).unwrap(), 1);

    let query_todo_list: Value = dot_get!(edges, "0.node");
    assert_eq!(dot_get!(query_todo_list, "id", String), todo_list_id);
    assert_eq!(
        dot_get!(query_todo_list, "title", String),
        dot_get!(todo_list, "title", String)
    );

    let query_todo_list0: Value = dot_get!(query_todo_list, "todos.edges.0.node");
    assert_eq!(
        dot_get!(query_todo_list0, "id", String),
        dot_get!(todos[0], "id", String)
    );
    assert_eq!(
        dot_get!(query_todo_list0, "title", String),
        dot_get!(todos[0], "title", String)
    );

    let query_todo_list1: Value = dot_get!(query_todo_list, "todos.edges.1.node");
    assert_eq!(
        dot_get!(query_todo_list1, "id", String),
        dot_get!(todos[1], "id", String)
    );
    assert_eq!(
        dot_get!(query_todo_list1, "title", String),
        dot_get!(todos[1], "title", String)
    );

    //
    // UPDATE
    //
    let response = client
        .gql::<Value>(DEFAULT_UPDATE)
        .variables(json!({ "id": todo_list_id }))
        .send();
    let updated_todo_list: Value = dot_get!(response, "data.todoListUpdate.todoList");
    assert_eq!(dot_get!(updated_todo_list, "title", String), "Updated Title");
}
