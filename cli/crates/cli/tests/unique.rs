mod utils;

use json_dotpath::DotPaths;
use serde_json::{json, Value};
use utils::consts::{
    UNIQUE_CREATE_MUTATION, UNIQUE_DELETE_MUTATION, UNIQUE_PAGINATED_QUERY, UNIQUE_QUERY, UNIQUE_QUERY_BY_NAME,
    UNIQUE_SCHEMA, UNIQUE_UPDATE_BY_NAME_MUTATION, UNIQUE_UPDATE_MUTATION, UNIQUE_UPDATE_UNIQUE_BY_NAME_MUTATION,
    UNIQUE_UPDATE_UNIQUE_MUTATION,
};
use utils::{client::Client, environment::Environment};

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

    assert!(errors.is_none(), "Expected no errors, but got {errors:?}");

    let response = client.gql::<Value>(
        json!({ "query": UNIQUE_UPDATE_MUTATION, "variables": { "id": first_author_id, "age": 40 } }).to_string(),
    );

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_none(), "Expected no errors, but got {errors:?}");

    let response =
        client.gql::<Value>(json!({ "query": UNIQUE_QUERY_BY_NAME, "variables": {"name": "1" } }).to_string());

    let query_author_id: String = dot_get!(response, "data.author.id");
    let query_author_age: usize = dot_get!(response, "data.author.age");
    let query_author_name: String = dot_get!(response, "data.author.name");

    assert_eq!(query_author_id, first_author_id);
    assert_eq!(query_author_age, 40);

    client.gql::<Value>(
        json!({ "query": UNIQUE_UPDATE_BY_NAME_MUTATION, "variables": { "name": query_author_name, "age": 50 } })
            .to_string(),
    );

    let response = client
        .gql::<Value>(json!({ "query": UNIQUE_QUERY_BY_NAME, "variables": {"name": query_author_name } }).to_string());

    let query_author_id: String = dot_get!(response, "data.author.id");
    let query_author_age: usize = dot_get!(response, "data.author.age");

    assert_eq!(query_author_id, first_author_id);
    assert_eq!(query_author_age, 50);

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

    client.gql::<Value>(
        json!({ "query": UNIQUE_UPDATE_UNIQUE_BY_NAME_MUTATION, "variables": { "queryName": query_author_name, "name": "4" } })
            .to_string(),
    );

    let response =
        client.gql::<Value>(json!({ "query": UNIQUE_QUERY_BY_NAME, "variables": { "name": "4" } }).to_string());

    let query_author_id: String = dot_get!(response, "data.author.id");
    let query_author_name: String = dot_get!(response, "data.author.name");

    assert_eq!(query_author_id, first_author_id);
    assert_eq!(query_author_name, "4");

    let response = client
        .gql::<Value>(json!({ "query": UNIQUE_DELETE_MUTATION, "variables": { "id": first_author_id } }).to_string());

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_none(), "Expected no errors, but got {errors:?}");

    let response = client
        .gql::<Value>(json!({ "query": UNIQUE_CREATE_MUTATION, "variables": { "name": "1", "age": 30 } }).to_string());

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_none(), "Expected no errors, but got {errors:?}");

    let response =
        client.gql::<Value>(json!({ "query": UNIQUE_QUERY, "variables": { "id": first_author_id } }).to_string());

    let first_author: Option<Value> = response.dot_get("data.author").unwrap();

    assert!(first_author.is_none());
}

pub const ACCOUNT_CREATE_MUTATION: &str = include_str!("graphql/unique/multiple-field/account-create.graphql");
pub const ACCOUNT_UPDATE_MUTATION: &str = include_str!("graphql/unique/multiple-field/account-update.graphql");
pub const ACCOUNT_DELETE_MUTATION: &str = include_str!("graphql/unique/multiple-field/account-delete.graphql");
pub const ACCOUNT_QUERY_ONE: &str = include_str!("graphql/unique/multiple-field/account-query-one.graphql");
pub const ACCOUNT_QUERY_PAGINATED: &str = include_str!("graphql/unique/multiple-field/account-query-paginated.graphql");

#[test]
fn unique_with_multiple_fields() {
    let mut env = Environment::init(4017);

    env.grafbase_init();

    env.write_schema(UNIQUE_SCHEMA);

    env.grafbase_dev();

    let client = env.create_client();

    client.poll_endpoint(30, 300);

    let email_and_provider_id = json!({"email": "test@example.com", "providerId": "1"});

    let response = client.create_account(&email_and_provider_id);

    assert_eq!(response.dot_get::<Value>("errors").unwrap(), None);

    let account_id = dot_get!(response, "data.accountCreate.account.id", String);

    let response = client.account_collection();

    assert_eq!(dot_get!(response, "data.accountCollection.edges", Vec<Value>).len(), 1);
    assert_eq!(
        dot_get!(response, "data.accountCollection.edges.0.node.id", String),
        account_id
    );
    assert_eq!(
        dot_get!(response, "data.accountCollection.edges.0.node.email", String),
        "test@example.com"
    );
    assert_eq!(
        dot_get!(response, "data.accountCollection.edges.0.node.providerId", String),
        "1"
    );

    let response = client.get_account(&json!({ "emailAndProviderId": email_and_provider_id }));

    assert_eq!(dot_get!(response, "data.account.id", String), account_id);
    assert_eq!(dot_get!(response, "data.account.email", String), "test@example.com");
    assert_eq!(dot_get!(response, "data.account.providerId", String), "1");

    // Create an account with the same data as above - it should fail.
    let response = client.create_account(&email_and_provider_id);

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_some());

    let error: String = dot_get!(response, "errors.0.message");

    assert!(error.contains("value"));
    assert!(error.contains("field"));

    // Create an account with a different providerId - it should work
    let response = client.create_account(&json!(
        { "email": "test@example.com", "providerId": "2" }
    ));

    let errors: Option<Value> = response.dot_get("errors").unwrap();
    assert!(errors.is_none(), "Expected no errors, but got {errors:?}");

    // Attempt to do an update that will clash
    let response = client.update_account(&json!({
        "by": {"id": account_id},
        "input": {"providerId": "2"}
    }));
    assert!(
        response.dot_get::<Value>("errors").unwrap().is_some(),
        "Expected errors, but got none"
    );

    // Attempt to do an update that won't clash
    let response = client.update_account(&json!({
        "by": {"emailAndProviderId": email_and_provider_id},
        "input": {"providerId": "3"}
    }));

    assert!(
        response.dot_get::<Value>("errors").unwrap().is_some(),
        "Expected no errors, but got {errors:?}"
    );

    let response = client.delete_account(&json!({ "emailAndProviderId": &email_and_provider_id }));

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_none(), "Expected no errors, but got {errors:?}");

    let response = client.create_account(&email_and_provider_id);

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_none(), "Expected no errors, but got {errors:?}");

    let response = client.get_account(&json!({ "id": account_id }));

    let first_author: Option<Value> = response.dot_get("data.author").unwrap();

    assert!(first_author.is_none());
}

impl Client {
    fn get_account(&self, by: &Value) -> Value {
        self.gql(
            json!({
                "query": ACCOUNT_QUERY_ONE,
                "variables": { "by": by }
            })
            .to_string(),
        )
    }

    fn account_collection(&self) -> Value {
        self.gql(json!({ "query": ACCOUNT_QUERY_PAGINATED }).to_string())
    }

    fn create_account(&self, input: &Value) -> Value {
        self.gql(
            json!({
                "query": ACCOUNT_CREATE_MUTATION,
                "variables": {"input": input}
            })
            .to_string(),
        )
    }

    fn update_account(&self, vars: &Value) -> Value {
        self.gql(
            json!({
                "query": ACCOUNT_CREATE_MUTATION,
                "variables": vars
            })
            .to_string(),
        )
    }

    fn delete_account(&self, by: &Value) -> Value {
        self.gql(
            json!({
                "query": ACCOUNT_DELETE_MUTATION,
                "variables": { "by": by }
            })
            .to_string(),
        )
    }
}
