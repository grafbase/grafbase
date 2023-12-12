#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::GraphType;
use serde_json::{json, Value};
use utils::consts::{DEFAULT_CREATE, DEFAULT_QUERY, DEFAULT_SCHEMA, DEFAULT_UPDATE};
use utils::environment::Environment;

#[rstest::rstest]
#[case::dev(true)]
#[case::start(false)]
fn dev(#[case] use_dev: bool) {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(DEFAULT_SCHEMA);
    if use_dev {
        env.grafbase_dev();
    } else {
        env.grafbase_start();
    }
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    //
    // CREATE
    //
    let response = client.gql::<Value>(DEFAULT_CREATE).send();
    let todo_list: Value = dot_get!(response, "data.todoListCreate.todoList");
    let todo_list_id: String = dot_get!(todo_list, "id");
    assert!(!todo_list_id.is_empty());
    assert_eq!(dot_get!(todo_list, "status", String), "BACKLOG");
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
        .variables(json!({ "id": todo_list_id, "input": {
            "title": "Sweet and Sour",
            "tags": ["Plum"],
            "likes": { "set": 10 }
        }}))
        .send();
    let updated_todo_list: Value = dot_get!(response, "data.todoListUpdate.todoList");
    assert_eq!(dot_get!(updated_todo_list, "title", String), "Sweet and Sour");
    assert_eq!(dot_get!(updated_todo_list, "likes", i32), 10);
    assert_eq!(dot_get!(updated_todo_list, "tags", Vec<String>), vec!["Plum"]);

    let response = client
        .gql::<Value>(DEFAULT_UPDATE)
        .variables(json!({ "id": todo_list_id, "input": { "tags": Value::Null, "status": "IN_PROGRESS" } }))
        .send();
    let updated_todo_list: Value = dot_get!(response, "data.todoListUpdate.todoList");
    assert_eq!(dot_get!(updated_todo_list, "title", String), "Sweet and Sour");
    assert_eq!(dot_get!(updated_todo_list, "likes", i32), 10);
    assert_eq!(dot_get!(updated_todo_list, "status", String), "IN_PROGRESS");
    assert_eq!(dot_get_opt!(updated_todo_list, "tags", Vec<String>), None);
}

#[cfg(not(target_os = "windows"))]
#[test]
fn dev_with_esm_project() {
    let mut env = Environment::init();

    env.write_json_file_to_project(
        "package.json",
        &json!({
          "name": "test",
          "version": "1.0.0",
          "description": "",
          "type": "module", // This is the important part for this test
          "main": "index.js",
          "keywords": [],
          "author": "",
          "license": "ISC"
        }),
    );
    env.grafbase_init(GraphType::Single);
    env.prepare_ts_config_dependencies();

    env.grafbase_dev();

    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);
}
