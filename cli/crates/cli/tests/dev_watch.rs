mod cargo_bin;
mod client;
mod consts;
mod macros;
mod utils;

use crate::cargo_bin::cargo_bin;
use crate::client::Client;
use crate::consts::{DEFAULT_QUERY, DEFAULT_SCHEMA, UPDATED_MUTATION, UPDATED_QUERY, UPDATED_SCHEMA};
use crate::utils::kill_with_children;
use duct::cmd;
use json_dotpath::DotPaths;
use serde_json::{json, Value};
use std::io::Write;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};
use tempfile::tempdir;

#[test]
fn dev_watch() {
    let port = 4001;
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

    let command = cmd!(cargo_bin("grafbase"), "dev", "--port", port.to_string())
        .dir(&temp_dir.path())
        .start()
        .unwrap();

    let client = Client::new(endpoint.clone());

    client.poll_endpoint(30, 300);

    let response = client.gql::<Value>(json!({ "query": DEFAULT_QUERY }).to_string());

    let todo_list_collection: Value = dot_get!(response, "data.todoListCollection.edges");

    assert!(todo_list_collection.is_array());
    assert!(!todo_list_collection.dot_has_checked("<").unwrap());

    let mut file = fs::OpenOptions::new().append(true).open(&schema_path).unwrap();

    file.write_all(format!("\n{UPDATED_SCHEMA}").as_bytes()).unwrap();

    file.sync_all().unwrap();

    drop(file);

    // wait for change to be picked up
    if env::var("CI").is_ok() {
        sleep(Duration::from_secs(4));
    } else {
        sleep(Duration::from_secs(2));
    }

    client.poll_endpoint(30, 300);

    client.gql::<Value>(json!({ "query": UPDATED_MUTATION }).to_string());

    let response = client.gql::<Value>(json!({ "query": UPDATED_QUERY }).to_string());
    let author_id: String = dot_get!(response, "data.authorCollection.edges.0.node.id");

    assert!(author_id.starts_with("Author#"));

    kill_with_children(*command.pids().first().unwrap());
}
