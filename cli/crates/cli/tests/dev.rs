mod cargo_bin;
mod client;
mod consts;
mod macros;
mod utils;

use crate::cargo_bin::cargo_bin;
use crate::client::Client;
use crate::consts::{DEFAULT_MUTATION, DEFAULT_QUERY, DEFAULT_SCHEMA};
use crate::utils::kill_with_children;
use duct::cmd;
use serde_json::{json, Value};
use std::process::Command;
use std::{env, fs};
use tempfile::tempdir;

#[test]
fn dev() {
    let port = 4000;
    let temp_dir = tempdir().unwrap();
    let endpoint = format!("http://127.0.0.1:{port}/graphql");

    env::set_current_dir(temp_dir.path()).unwrap();

    let schema_path = temp_dir.path().join("grafbase").join("schema.graphql");

    Command::new(cargo_bin("grafbase"))
        .args(&["init"])
        .current_dir(&temp_dir.path())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    fs::write(&schema_path, DEFAULT_SCHEMA).unwrap();

    let command = cmd!(
        cargo_bin("grafbase"),
        "dev",
        "--disable-watch",
        "--port",
        port.to_string()
    )
    .dir(&temp_dir.path())
    .start()
    .unwrap();

    let client = Client::new(endpoint);

    // wait for node to be ready
    client.poll_endpoint(30, 300);

    client.gql::<Value>(json!({ "query": DEFAULT_MUTATION }).to_string());

    let response = client.gql::<Value>(json!({ "query": DEFAULT_QUERY }).to_string());

    let todo_list: Value = dot_get!(response, "data.todoListCollection.edges.0.node");

    let todo_list_id: String = dot_get!(todo_list, "id");

    let first_todo_id: String = dot_get!(todo_list, "todos.0.id");

    assert!(todo_list_id.starts_with("TodoList#"));
    assert!(first_todo_id.starts_with("Todo#"));

    kill_with_children(*command.pids().first().unwrap());
}
