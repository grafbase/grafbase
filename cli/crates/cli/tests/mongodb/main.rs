#![allow(unused_crate_dependencies)]

mod create_many;
mod create_one;
mod delete_many;
mod delete_one;
mod find_many;
mod find_one;
mod update_many;
mod update_one;

#[path = "../utils/mod.rs"]
mod utils;

use std::{fmt, net::SocketAddr};

use backend::project::GraphType;
use indoc::formatdoc;
use reqwest::header::USER_AGENT;
use serde_json::{json, Value};
use utils::environment::Environment;
use wiremock::{
    matchers::{body_json, header, method, path},
    Mock, MockServer, ResponseTemplate, Times,
};

const MONGODB_PATH: &str = "/app/data-asdf/endpoint/data/v1";
const MONGODB_API_KEY: &str = "FAKE KEY";
const MONGODB_DATA_SOURCE: &str = "grafbase";
const MONGODB_DATABASE: &str = "test";
const MONGODB_CONNECTOR: &str = "mongo";

type JsonMap = serde_json::Map<String, Value>;

struct Server {
    action: &'static str,
    config: String,
    server: MockServer,
    request: Value,
    response: ResponseTemplate,
    expected_requests: u64,
}

impl Server {
    /// Construct a mock server to catch a findOne query.
    ///
    /// ## Parameters
    ///
    /// - config: the models and types as SDL
    /// - collection: the collection we're expected to query
    /// - body: the expected request body we send to `MongoDB`
    ///
    /// [docs](https://www.mongodb.com/docs/atlas/api/data-api-resources/#find-a-single-document)
    pub async fn find_one(config: impl fmt::Display, collection: &'static str, body: Value) -> Self {
        let server = MockServer::start().await;
        let request = body.as_object().cloned().unwrap();
        let response = ResponseTemplate::new(200).set_body_json(json!({ "document": null }));

        Self {
            action: "findOne",
            config: Self::merge_config(config, server.address()),
            server,
            request: Self::create_request(collection, request),
            response,
            expected_requests: 1,
        }
    }

    /// Construct a mock server to catch a findMany query.
    ///
    /// ## Parameters
    ///
    /// - config: the models and types as SDL
    /// - collection: the collection we're expected to query
    /// - filter: the expected filter we send to `MongoDB`
    /// - projection: the expected projection we send to `MongoDB`
    /// - response: the mock response we expect `MongoDB` to send us
    ///
    /// [docs](https://www.mongodb.com/docs/atlas/api/data-api-resources/#find-a-single-document)
    pub async fn find_many(config: impl fmt::Display, collection: &'static str, body: Value) -> Self {
        let server = MockServer::start().await;
        let request = body.as_object().cloned().unwrap();

        let response = ResponseTemplate::new(200).set_body_json(json!({
            "documents": []
        }));

        Self {
            action: "find",
            config: Self::merge_config(config, server.address()),
            server,
            request: Self::create_request(collection, request),
            response,
            expected_requests: 1,
        }
    }

    /// Construct a mock server to catch a createOne query.
    ///
    /// ## Parameters
    ///
    /// - config: the models and types as SDL
    /// - collection: the collection we're expected to query
    /// - document: the expected document we send to `MongoDB`
    /// - response: the mock response we expect `MongoDB` to send us
    ///
    /// [docs](https://www.mongodb.com/docs/atlas/api/data-api-resources/#insert-a-single-document)
    pub async fn create_one(config: impl fmt::Display, collection: &'static str, body: Value) -> Self {
        let server = MockServer::start().await;
        let request = body.as_object().cloned().unwrap();

        let response = ResponseTemplate::new(200).set_body_json(json!({
            "insertedId": "5ca4bbc7a2dd94ee5816238d"
        }));

        Self {
            action: "insertOne",
            config: Self::merge_config(config, server.address()),
            server,
            request: Self::create_request(collection, request),
            response,
            expected_requests: 1,
        }
    }

    /// Construct a mock server to catch a createMany  query.
    ///
    /// ## Parameters
    ///
    /// - config: the models and types as SDL
    /// - collection: the collection we're expected to query
    /// - documents: the expected documents we send to `MongoDB`
    /// - response: the mock response we expect `MongoDB` to send us
    ///
    /// [docs](https://www.mongodb.com/docs/atlas/app-services/data-api/openapi/#operation/insertMany)
    pub async fn create_many(config: impl fmt::Display, collection: &'static str, body: Value) -> Self {
        let server = MockServer::start().await;
        let request = body.as_object().cloned().unwrap();

        let response = ResponseTemplate::new(200).set_body_json(json!({
            "insertedIds": ["5ca4bbc7a2dd94ee5816238d", "5ca4bbc7a2dd94ee5816238e"]
        }));

        Self {
            action: "insertMany",
            config: Self::merge_config(config, server.address()),
            server,
            request: Self::create_request(collection, request),
            response,
            expected_requests: 1,
        }
    }

    /// Construct a mock server to catch a deleteOne query.
    ///
    /// ## Parameters
    ///
    /// - config: the models and types as SDL
    /// - collection: the collection we're expected to query
    /// - filter: the expected filter we send to `MongoDB`
    /// - response: the mock response we expect `MongoDB` to send us
    ///
    /// [docs](https://www.mongodb.com/docs/atlas/api/data-api-resources/#delete-a-single-document)
    async fn delete_one(config: impl fmt::Display, collection: &'static str, body: Value) -> Self {
        let server = MockServer::start().await;
        let request = body.as_object().cloned().unwrap();

        let response = ResponseTemplate::new(200).set_body_json(json!({
            "deletedCount": 1
        }));

        Self {
            action: "deleteOne",
            config: Self::merge_config(config, server.address()),
            server,
            request: Self::create_request(collection, request),
            response,
            expected_requests: 1,
        }
    }

    /// Construct a mock server to catch a updateOne query.
    ///
    /// ## Parameters
    ///
    /// - config: the models and types as SDL
    /// - collection: the collection we're expected to query
    /// - filter: the expected filter we send to `MongoDB`
    /// - update: the expected update statement we send to `MongoDB`
    /// - response: the mock response we expect `MongoDB` to send us
    ///
    /// [docs](https://www.mongodb.com/docs/atlas/app-services/data-api/openapi/#operation/updateOne)
    async fn update_one(config: impl fmt::Display, collection: &'static str, body: Value) -> Self {
        let server = MockServer::start().await;
        let request = body.as_object().cloned().unwrap();

        let response = ResponseTemplate::new(200).set_body_json(json!({
            "matchedCount": 1,
            "modifiedCount": 1,
        }));

        Self {
            action: "updateOne",
            config: Self::merge_config(config, server.address()),
            server,
            request: Self::create_request(collection, request),
            response,
            expected_requests: 1,
        }
    }

    /// Construct a mock server to catch a updateMany query.
    ///
    /// ## Parameters
    ///
    /// - config: the models and types as SDL
    /// - collection: the collection we're expected to query
    /// - filter: the expected filter we send to `MongoDB`
    /// - update: the expected update statement we send to `MongoDB`
    /// - response: the mock response we expect `MongoDB` to send us
    ///
    /// [docs](https://www.mongodb.com/docs/atlas/app-services/data-api/openapi/#operation/updateMany)
    async fn update_many(config: impl fmt::Display, collection: &'static str, body: Value) -> Self {
        let server = MockServer::start().await;
        let request = body.as_object().cloned().unwrap();

        let response = ResponseTemplate::new(200).set_body_json(json!({
            "matchedCount": 1,
            "modifiedCount": 1,
        }));

        Self {
            action: "updateMany",
            config: Self::merge_config(config, server.address()),
            server,
            request: Self::create_request(collection, request),
            response,
            expected_requests: 1,
        }
    }

    /// Construct a mock server to catch a deleteMany query.
    ///
    /// ## Parameters
    ///
    /// - config: the models and types as SDL
    /// - collection: the collection we're expected to query
    /// - filter: the expected filter we send to `MongoDB`
    /// - response: the mock response we expect `MongoDB` to send us
    ///
    /// [docs](https://www.mongodb.com/docs/atlas/app-services/data-api/openapi/#operation/deleteMany)
    async fn delete_many(config: impl fmt::Display, collection: &'static str, body: Value) -> Self {
        let server = MockServer::start().await;
        let request = body.as_object().cloned().unwrap();

        let response = ResponseTemplate::new(200).set_body_json(json!({
            "deletedCount": 2
        }));

        Self {
            action: "deleteMany",
            config: Self::merge_config(config, server.address()),
            server,
            request: Self::create_request(collection, request),
            response,
            expected_requests: 1,
        }
    }

    /// Send a query or mutation to the server, returning JSON response.
    pub async fn request(&self, request: &str) -> Value {
        let request_path = format!("{MONGODB_PATH}/action/{}", self.action);

        Mock::given(method("POST"))
            .and(path(&request_path))
            .and(header("apiKey", MONGODB_API_KEY))
            .and(header(USER_AGENT, "Grafbase"))
            .and(body_json(&self.request))
            .respond_with(self.response.clone())
            .expect(Times::from(self.expected_requests))
            .mount(&self.server)
            .await;

        let mut env = Environment::init_async().await;
        env.grafbase_init(GraphType::Single);
        env.write_schema(&self.config);
        env.set_variables([("API_KEY", "BLAH")]);
        env.grafbase_dev_watch();

        let client = env.create_async_client().with_api_key();

        client.poll_endpoint(30, 300).await;

        let response = client.gql(request).await;
        self.debug_received_requests().await;

        response
    }

    /// Prints all received requests for debugging.
    pub async fn debug_received_requests(&self) {
        let requests = self.server.received_requests().await.unwrap();

        println!("# Captured requests");

        for request in requests {
            let body: Value = request.body_json().unwrap();
            println!("## URL");
            println!("{}", request.url);

            println!("## Headers");
            for (name, value) in request.headers {
                println!("- {name:?}: {value:?}");
            }

            println!("## Body");
            println!("{}", serde_json::to_string_pretty(&body).unwrap());
        }
    }

    fn merge_config(config: impl fmt::Display, address: &SocketAddr) -> String {
        formatdoc!(
            r#"
            extend schema
              @mongodb(
                name: "{MONGODB_CONNECTOR}",
                namespace: false,
                url: "http://{address}{MONGODB_PATH}"
                apiKey: "{MONGODB_API_KEY}"
                dataSource: "{MONGODB_DATA_SOURCE}"
                database: "{MONGODB_DATABASE}"
              )

            {config} 
        "#
        )
    }

    fn create_request(collection: &'static str, data: JsonMap) -> Value {
        let mut request = JsonMap::new();
        request.insert("dataSource".to_string(), MONGODB_DATA_SOURCE.into());
        request.insert("database".to_string(), MONGODB_DATABASE.into());
        request.insert("collection".to_string(), collection.into());

        request.extend(data);

        Value::Object(request)
    }
}
