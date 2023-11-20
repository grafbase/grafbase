#![allow(unused_crate_dependencies, clippy::panic)]

use std::time::Duration;

use backend::project::ConfigType;
use headers::{CacheControl, HeaderMapExt};

use serde_json::Value;
use utils::{async_client::AsyncClient, environment::Environment};

mod utils;

const GRAFBASE_CACHE_HEADER: &str = "x-grafbase-cache";

fn header<'r>(response: &'r reqwest::Response, name: &'static str) -> Option<&'r str> {
    response.headers().get(name).map(|header| header.to_str().unwrap())
}

#[tokio::test(flavor = "multi_thread")]
async fn global_caching() {
    let mut env = Environment::init_async().await;
    let client = start_grafbase(
        &mut env,
        r#"
            extend schema @cache(rules: [{maxAge: 60, types: "Query"}])

            type Post @model {
                test: String!
            }
        "#,
    )
    .await;

    let call = || async {
        client
            .gql::<Value>("query {postCollection(first: 10) {edges {node {test}}}}")
            .into_reqwest_builder()
            .send()
            .await
            .unwrap()
    };

    let response = call().await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("MISS"));
    assert_eq!(
        response.headers().typed_get::<CacheControl>(),
        Some(CacheControl::new().with_public().with_max_age(Duration::from_secs(60)))
    );

    let response = call().await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("HIT"));
    assert_eq!(response.headers().typed_get::<CacheControl>(), None);
}

#[tokio::test(flavor = "multi_thread")]
async fn model_and_field_caching() {
    let mut env = Environment::init_async().await;
    let client = start_grafbase(
        &mut env,
        r"
            type Model @model @cache(maxAge: 20) {
                test: String!
            }

            type ModelAndField @model @cache(maxAge: 30) {
                test: String! @cache(maxAge: 10)
            }
        ",
    )
    .await;

    let call = |query: &'static str| async {
        let query = query.to_string();
        client.gql::<Value>(query).into_reqwest_builder().send().await.unwrap()
    };

    // model caching
    let query = r"query {modelCollection(first: 10) {edges {node {test}}}}";
    let response = call(query).await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("MISS"));
    assert_eq!(
        response.headers().typed_get::<CacheControl>(),
        Some(CacheControl::new().with_public().with_max_age(Duration::from_secs(20)))
    );

    let response = call(query).await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("HIT"));
    assert_eq!(response.headers().typed_get::<CacheControl>(), None);

    // field caching
    let query = r"query {modelAndFieldCollection(first: 10) {edges {node {test}}}}";
    let response = call(query).await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("MISS"));
    assert_eq!(
        response.headers().typed_get::<CacheControl>(),
        Some(CacheControl::new().with_public().with_max_age(Duration::from_secs(10)))
    );

    let response = call(query).await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("HIT"));
    assert_eq!(response.headers().typed_get::<CacheControl>(), None);
}
#[tokio::test(flavor = "multi_thread")]
async fn no_cache_on_non_cahced_field() {
    let mut env = Environment::init_async().await;
    let client = start_grafbase(
        &mut env,
        r"
            type Post @model {
                test: String!  @cache(maxAge: 20)
            }
        ",
    )
    .await;

    let call = || async {
        client
            .gql::<Value>("query {postCollection(first: 10) {edges {node {id}}}}")
            .into_reqwest_builder()
            .send()
            .await
            .unwrap()
    };

    let response = call().await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("BYPASS"));
    assert_eq!(response.headers().typed_get::<CacheControl>(), None);

    let response = call().await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("BYPASS"));
    assert_eq!(response.headers().typed_get::<CacheControl>(), None);
}

#[tokio::test(flavor = "multi_thread")]
async fn no_cache_on_mutations() {
    let mut env = Environment::init_async().await;
    let client = start_grafbase(
        &mut env,
        r"
            type Post @model @cache(maxAge: 30) {
                test: String! @unique @cache(maxAge: 10)
            }
        ",
    )
    .await;

    let call = || async {
        client
            .gql::<Value>(
                r#"
                mutation {
                    postCreate(input: {
                        test: "hello"
                    }) {
                        post {
                            id
                            test
                        }
                    }
                }
                "#,
            )
            .into_reqwest_builder()
            .send()
            .await
            .unwrap()
    };

    let response = call().await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("BYPASS"));
    assert_eq!(response.headers().typed_get::<CacheControl>(), None);

    let response = call().await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("BYPASS"));
    assert_eq!(response.headers().typed_get::<CacheControl>(), None);
}

#[tokio::test(flavor = "multi_thread")]
async fn no_cache_on_same_query_different_variables() {
    let mut env = Environment::init_async().await;
    let client = start_grafbase(
        &mut env,
        r"
            type Post @model @cache(maxAge: 30) {
                test: String! @unique @cache(maxAge: 10)
            }
        ",
    )
    .await;

    let call = |variables: serde_json::Value| async {
        client
            .gql::<Value>(
                r"
                query PostByTest($test: String!) {
                    post(by: { test: $test }) {
                        id
                        test
                    }
                }
                ",
            )
            .variables(variables)
            .into_reqwest_builder()
            .send()
            .await
            .unwrap()
    };
    let response = call(serde_json::json!({ "test": "hello" })).await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("MISS"));
    assert_eq!(
        response.headers().typed_get::<CacheControl>(),
        Some(CacheControl::new().with_public().with_max_age(Duration::from_secs(10)))
    );

    let response = call(serde_json::json!({ "test": "world" })).await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("MISS"));
    assert_eq!(
        response.headers().typed_get::<CacheControl>(),
        Some(CacheControl::new().with_public().with_max_age(Duration::from_secs(10)))
    );

    let response = call(serde_json::json!({ "test": "hello" })).await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("HIT"));
    assert_eq!(response.headers().typed_get::<CacheControl>(), None);

    let response = call(serde_json::json!({ "test": "world" })).await;
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), Some("HIT"));
    assert_eq!(response.headers().typed_get::<CacheControl>(), None);
}

#[tokio::test(flavor = "multi_thread")]
async fn no_cache_header_when_caching_is_not_used() {
    let mut env = Environment::init_async().await;
    let client = start_grafbase(
        &mut env,
        r"
            type Post @model {
                test: String!
            }
        ",
    )
    .await;

    let response = client
        .gql::<Value>("query {postCollection(first: 10) {edges {node {test}}}}")
        .into_reqwest_builder()
        .send()
        .await
        .unwrap();
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), None);
    assert_eq!(response.headers().typed_get::<CacheControl>(), None);
}

async fn start_grafbase(env: &mut Environment, schema: impl AsRef<str>) -> AsyncClient {
    env.grafbase_init(ConfigType::GraphQL);
    env.write_schema(schema);
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    client
}
