mod cargo_bin;
mod consts;
mod macros;
mod utils;

use crate::cargo_bin::cargo_bin;
use crate::consts::{
    RELATIONS_LINK_BLOG_TO_AUTHOR, RELATIONS_MUTATION, RELATIONS_QUERY, RELATIONS_SCHEMA,
    RELATIONS_UNLINK_BLOG_FROM_AUTHOR,
};
use crate::utils::{kill_with_children, poll_endpoint};
use common::environment::Environment;
use duct::cmd;
use serde_json::{json, Value};
use std::process::Command;
use std::{env, fs};
use tempfile::tempdir;

#[test]
fn relations() {
    let port = 4002;
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

    fs::write(&schema_path, RELATIONS_SCHEMA).unwrap();

    Environment::try_init().unwrap();

    let command = cmd!(cargo_bin("grafbase"), "dev", "--port", port.to_string())
        .dir(&temp_dir.path())
        .start()
        .unwrap();

    // wait for node to be ready
    poll_endpoint(&endpoint, 30, 300);

    let client = reqwest::blocking::Client::new();

    client
        .post(endpoint.clone())
        .body(json!({ "query": RELATIONS_MUTATION }).to_string())
        .send()
        .unwrap();

    let response = client
        .post(endpoint.clone())
        .body(json!({ "query": RELATIONS_QUERY }).to_string())
        .send()
        .unwrap()
        .json::<Value>()
        .unwrap();

    let blog: Value = dot_get!(response, "data.blogCollection.edges.0.node");
    let blog_id: String = dot_get!(blog, "id");
    let first_author_id: String = dot_get!(blog, "authors.0.id");
    let first_author_name: String = dot_get!(blog, "authors.0.name");
    let first_authors_blogs: Vec<Value> = dot_get!(response, "data.blogCollection.edges.0.node.authors.0.blogs");

    assert!(blog_id.starts_with("Blog#"));
    // latest first
    assert_eq!(first_author_name, "2");
    assert!(first_authors_blogs.is_empty());

    client
        .post(endpoint.clone())
        .body(
            json!({
                "query": RELATIONS_LINK_BLOG_TO_AUTHOR,
                "variables": { "id": first_author_id, "blogId": blog_id}
            })
            .to_string(),
        )
        .send()
        .unwrap();

    let response = client
        .post(endpoint.clone())
        .body(json!({ "query": RELATIONS_QUERY }).to_string())
        .send()
        .unwrap()
        .json::<Value>()
        .unwrap();

    let current_first_author_id: String = dot_get!(response, "data.blogCollection.edges.0.node.authors.0.id");
    let first_authors_first_blog_id: Value =
        dot_get!(response, "data.blogCollection.edges.0.node.authors.0.blogs.0.id");

    assert_eq!(current_first_author_id, first_author_id);
    assert_eq!(blog_id, first_authors_first_blog_id);
    assert_eq!(blog_id, first_authors_first_blog_id);

    client
        .post(endpoint.clone())
        .body(
            json!({
                "query": RELATIONS_UNLINK_BLOG_FROM_AUTHOR,
                "variables": { "id": first_author_id, "blogId": blog_id}
            })
            .to_string(),
        )
        .send()
        .unwrap();

    let response = client
        .post(endpoint)
        .body(json!({ "query": RELATIONS_QUERY }).to_string())
        .send()
        .unwrap()
        .json::<Value>()
        .unwrap();

    let current_first_author_id: String = dot_get!(response, "data.blogCollection.edges.0.node.authors.0.id");
    let first_authors_blogs: Vec<Value> = dot_get!(response, "data.blogCollection.edges.0.node.authors.0.blogs");

    assert_eq!(current_first_author_id, first_author_id);
    assert!(first_authors_blogs.is_empty());

    kill_with_children(*command.pids().first().unwrap());
}
