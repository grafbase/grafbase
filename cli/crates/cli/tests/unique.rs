mod utils;

use json_dotpath::DotPaths;
use serde_json::{json, Value};
use utils::consts::{
    UNIQUE_CREATE_MUTATION, UNIQUE_DELETE_MUTATION, UNIQUE_PAGINATED_QUERY, UNIQUE_QUERY, UNIQUE_QUERY_BY_NAME,
    UNIQUE_SCHEMA, UNIQUE_UPDATE_MUTATION, UNIQUE_UPDATE_UNIQUE_MUTATION,
};
use utils::environment::Environment;

#[test]
fn unique() {
    let mut env = Environment::init(4003);

    env.grafbase_init();

    env.write_schema(UNIQUE_SCHEMA);

    env.grafbase_dev();

    let client = env.create_client();

    client.poll_endpoint(30, 300);

    client
        .gql::<Value>(json!({ "query": UNIQUE_CREATE_MUTATION, "variables": { "name": "1", "age": 30 } }).to_string());

    let response = client.gql::<Value>(json!({ "query": UNIQUE_PAGINATED_QUERY }).to_string());

    let first_author_id: String = dot_get!(response, "data.authorCollection.edges.0.node.id");

    assert!(first_author_id.starts_with("author_"));

    let response =
        client.gql::<Value>(json!({ "query": UNIQUE_QUERY, "variables": { "id": first_author_id } }).to_string());

    let updated_at: String = dot_get!(response, "data.author.updatedAt");
    assert!(
        chrono::DateTime::parse_from_rfc3339(&updated_at).is_ok(),
        "{}",
        updated_at
    );

    let created_at: String = dot_get!(response, "data.author.createdAt");
    assert!(
        chrono::DateTime::parse_from_rfc3339(&created_at).is_ok(),
        "{}",
        created_at
    );

    let first_author_name: String = dot_get!(response, "data.author.name");
    assert_eq!(first_author_name, "1");

    let response = client
        .gql::<Value>(json!({ "query": UNIQUE_QUERY_BY_NAME, "variables": { "name": first_author_name } }).to_string());

    let first_query_author_id: String = dot_get!(response, "data.author.id");

    assert_eq!(first_query_author_id, first_author_id);

    let response = client
        .gql::<Value>(json!({ "query": UNIQUE_CREATE_MUTATION, "variables": { "name": "1", "age": 30 } }).to_string());

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_some());

    let error: String = dot_get!(response, "errors.0.message");

    assert!(error.contains("value"));
    assert!(error.contains("field"));

    let response = client
        .gql::<Value>(json!({ "query": UNIQUE_CREATE_MUTATION, "variables": { "name": "2", "age": 30 } }).to_string());

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_none());

    client.gql::<Value>(
        json!({ "query": UNIQUE_UPDATE_MUTATION, "variables": { "id": first_author_id, "age": 40 } }).to_string(),
    );

    let response =
        client.gql::<Value>(json!({ "query": UNIQUE_QUERY_BY_NAME, "variables": {"name": "1" } }).to_string());

    let query_author_id: String = dot_get!(response, "data.author.id");
    let query_author_age: usize = dot_get!(response, "data.author.age");

    assert_eq!(query_author_id, first_author_id);
    assert_eq!(query_author_age, 40);

    client.gql::<Value>(
        json!({ "query": UNIQUE_UPDATE_UNIQUE_MUTATION, "variables": { "id": first_author_id, "name": "3" } })
            .to_string(),
    );

    let response =
        client.gql::<Value>(json!({ "query": UNIQUE_QUERY_BY_NAME, "variables": { "name": "3" } }).to_string());

    let query_author_id: String = dot_get!(response, "data.author.id");
    let query_author_name: String = dot_get!(response, "data.author.name");

    assert_eq!(query_author_id, first_author_id);
    assert_eq!(query_author_name, "3");

    let response = client
        .gql::<Value>(json!({ "query": UNIQUE_DELETE_MUTATION, "variables": { "id": first_author_id } }).to_string());

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_none());

    let response = client
        .gql::<Value>(json!({ "query": UNIQUE_CREATE_MUTATION, "variables": { "name": "1", "age": 30 } }).to_string());

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_none());

    let response =
        client.gql::<Value>(json!({ "query": UNIQUE_QUERY, "variables": { "id": first_author_id } }).to_string());

    let first_author: Option<Value> = response.dot_get("data.author").unwrap();

    assert!(first_author.is_none());
}
