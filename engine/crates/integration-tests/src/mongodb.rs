use std::{fmt, future::Future, panic::AssertUnwindSafe, sync::Arc};

use engine::Response;
use futures::FutureExt;
use indoc::formatdoc;
use serde_json::json;

use crate::Engine;

pub(super) static DATA_API_URL: &str = "http://localhost:3000/app/data-test/endpoint/data/v1";

/// With a given schema, initialize Engine and provide a test API
/// to run queries and mutations against.
///
/// The provided closure should return a `Response`, which is then
/// serialized as JSON string as the return value.
///
/// The database is given a random name, and is dropped after the
/// closure is done.
///
/// This function doesn't namespace the Mongo models, but has them
/// directly in the root.
#[track_caller]
pub fn with_mongodb<D, F, U>(schema: D, test: F) -> String
where
    D: fmt::Display,
    F: FnOnce(TestApi) -> U,
    U: Future<Output = Response>,
{
    let database = super::random_name();
    let test_api = || async { TestApi::new(&database, schema).await };

    inner_mongodb(test_api, &database, test)
}

/// With a given schema, initialize Engine and provide a test API
/// to run queries and mutations against.
///
/// The provided closure should return a `Response`, which is then
/// serialized as JSON string as the return value.
///
/// The database is given a random name, and is dropped after the
/// closure is done.
///
/// This function adds the Mongo models to a namespace.
#[track_caller]
pub fn with_namespaced_mongodb<D, F, U>(namespace: &str, schema: D, test: F) -> String
where
    D: fmt::Display,
    F: FnOnce(TestApi) -> U,
    U: Future<Output = Response>,
{
    let database = super::random_name();
    let test_api = || async { TestApi::new_namespaced(&database, namespace, schema).await };

    inner_mongodb(test_api, &database, test)
}

#[track_caller]
fn inner_mongodb<G, O, A, T>(api: A, database: &str, test: G) -> String
where
    G: FnOnce(TestApi) -> O,
    O: Future<Output = Response>,
    A: FnOnce() -> T,
    T: Future<Output = TestApi>,
{
    super::runtime().block_on(async {
        let api = api().await;
        let response = AssertUnwindSafe(test(api.clone())).catch_unwind().await;

        api.inner
            .client
            .post(format!("{DATA_API_URL}/action/dropDatabase"))
            .json(&json!({
                "dataSource": "grafbase",
                "database": database
            }))
            .send()
            .await
            .expect("Error when dropping the database.");

        let response = response.expect("Error in test execution.");
        serde_json::to_string_pretty(&response.to_graphql_response()).unwrap()
    })
}

struct Inner {
    engine: Engine,
    client: reqwest::Client,
    database: String,
}

#[derive(Clone)]
pub struct TestApi {
    inner: Arc<Inner>,
}

impl TestApi {
    async fn new(database: &str, schema: impl fmt::Display) -> Self {
        let schema = formatdoc! {r#"
            extend schema
              @mongodb(
                 name: "test",
                 apiKey: "TEST"
                 url: "{DATA_API_URL}"
                 dataSource: "TEST"
                 database: "{database}"
                 namespace: false
              )

            {schema}
        "#};

        Self::new_inner(database, schema).await
    }

    async fn new_namespaced(database: &str, namespace: &str, schema: impl fmt::Display) -> Self {
        let schema = formatdoc!(
            r#"
            extend schema
              @mongodb(
                 name: "{namespace}",
                 apiKey: "TEST"
                 url: "{DATA_API_URL}"
                 dataSource: "TEST"
                 database: "{database}"
                 namespace: true
              )

            {schema}
        "#
        );

        Self::new_inner(database, schema).await
    }

    async fn new_inner(database: &str, schema: String) -> Self {
        let engine = Engine::new(schema).await;
        let client = reqwest::Client::new();

        Self {
            inner: Arc::new(Inner {
                engine,
                client,
                database: database.to_string(),
            }),
        }
    }

    /// Execute a GraphQL query or mutation against the database.
    pub async fn execute(&self, operation: impl AsRef<str>) -> Response {
        self.inner.engine.execute(operation.as_ref()).await
    }

    pub fn engine(&self) -> &Engine {
        &self.inner.engine
    }

    /// Insert a document directly to the underlying database.
    pub async fn insert_one(&self, collection: &str, document: serde_json::Value) -> MongoDBCreateOneResponse {
        let request = MongoDBCreateOneRequest {
            data_source: "TEST",
            database: &self.inner.database,
            collection,
            document,
        };

        let url = format!("{DATA_API_URL}/action/insertOne");
        let res = self.inner.client.post(url).json(&request).send().await.unwrap();

        res.json().await.unwrap()
    }

    /// Insert multiple documents directly to the underlying database.
    pub async fn insert_many(&self, collection: &str, documents: serde_json::Value) -> MongoDBCreateManyResponse {
        let request = MongoDBCreateManyRequest {
            data_source: "TEST",
            database: &self.inner.database,
            collection,
            documents,
        };

        let url = format!("{DATA_API_URL}/action/insertMany");
        let res = self.inner.client.post(url).json(&request).send().await.unwrap();

        res.json().await.unwrap()
    }

    /// Load all documents from the given collection
    pub async fn fetch_all(&self, collection: &str, projection: serde_json::Value) -> MongoDBFetchAllResponse {
        let request = MongoDBFetchAllRequest {
            data_source: "TEST",
            database: &self.inner.database,
            collection,
            projection,
            filter: json!({}),
        };

        let url = format!("{DATA_API_URL}/action/find");
        let res = self.inner.client.post(url).json(&request).send().await.unwrap();

        res.json().await.unwrap()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MongoDBCreateOneRequest<'a> {
    data_source: &'static str,
    database: &'a str,
    collection: &'a str,
    document: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MongoDBCreateManyRequest<'a> {
    data_source: &'static str,
    database: &'a str,
    collection: &'a str,
    documents: serde_json::Value,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MongoDBCreateOneResponse {
    pub inserted_id: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MongoDBCreateManyResponse {
    pub inserted_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MongoDBFetchAllRequest<'a> {
    data_source: &'static str,
    database: &'a str,
    collection: &'a str,
    projection: serde_json::Value,
    filter: serde_json::Value,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MongoDBFetchAllResponse {
    pub documents: Vec<serde_json::Value>,
}
