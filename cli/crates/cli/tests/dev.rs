mod cargo_bin;
mod consts;
mod types;
mod utils;

use crate::cargo_bin::cargo_bin;
use crate::consts::{DEFAULT_MUTATION, DEFAULT_QUERY, DEFAULT_SCHEMA};
use crate::types::TodoListCollectionResponse;
use crate::utils::{kill_with_children, poll_endpoint};
use common::environment::Environment;
use duct::cmd;
use serde_json::json;
use std::process::Command;
use std::{env, fs};
use tempfile::tempdir;

#[test]
fn sanity() {
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

    Environment::try_init().unwrap();

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

    // wait for node to be ready
    poll_endpoint(&endpoint, 30, 300);

    let client = reqwest::blocking::Client::new();

    client
        .post(endpoint.clone())
        .body(json!({ "query": DEFAULT_MUTATION }).to_string())
        .send()
        .unwrap();

    let response = client
        .post(endpoint)
        .body(json!({ "query": DEFAULT_QUERY }).to_string())
        .send()
        .unwrap()
        .json::<TodoListCollectionResponse>()
        .unwrap();

    let todo_list = response.data.todo_list_collection.edges.first().unwrap().node.clone();

    assert!(todo_list.id.starts_with("TodoList#"));
    assert!(todo_list.todos.first().unwrap().id.starts_with("Todo#"));

    kill_with_children(*command.pids().first().unwrap());
}
