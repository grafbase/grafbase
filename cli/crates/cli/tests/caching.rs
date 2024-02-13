#![allow(unused_crate_dependencies, clippy::panic)]

use std::{fmt::Display, time::Duration};

use backend::project::GraphType;
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
    env.write_resolver(
        "post.js",
        r"
        export default function Resolver(parent, args, context, info) {
            return {
                title: (Math.random() + 1).toString(36).substring(7)
            }
        }
    ",
    );
    let client = start_grafbase(
        &mut env,
        r#"
            extend schema @cache(rules: [{maxAge: 60, types: "Query"}])

            type Query {
                post: Post! @resolver(name: "post")
            }

            type Post {
                title: String!
            }
        "#,
    )
    .await;

    let call = || async {
        let response = client
            .gql::<Value>("query { post { title } }")
            .into_reqwest_builder()
            .send()
            .await
            .unwrap();
        (
            response.headers().clone(),
            response.json::<serde_json::Value>().await.unwrap(),
        )
    };

    let (headers, content) = call().await;
    assert_eq!(
        headers.get(GRAFBASE_CACHE_HEADER).map(|v| v.to_str().unwrap()),
        Some("MISS")
    );
    assert_eq!(
        headers.typed_get::<CacheControl>(),
        Some(CacheControl::new().with_public().with_max_age(Duration::from_secs(60)))
    );

    let (headers, cached_content) = call().await;
    assert_eq!(cached_content, content);
    assert_eq!(
        headers.get(GRAFBASE_CACHE_HEADER).map(|v| v.to_str().unwrap()),
        Some("HIT")
    );
    assert_eq!(headers.typed_get::<CacheControl>(), None);
}

#[ignore]
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
async fn no_cache_on_non_cached_field() {
    let mut env = Environment::init_async().await;
    env.write_resolver(
        "post.js",
        r"
        export default function Resolver(parent, args, context, info) {
            return {
                title: (Math.random() + 1).toString(36).substring(7),
                author: (Math.random() + 1).toString(36).substring(7)
            }
        }
    ",
    );
    let client = start_grafbase(
        &mut env,
        r#"
            type Query {
                post: Post! @resolver(name: "post")
            }

            type Post {
                title: String! @cache(maxAge: 10)
                author: String!
            }
        "#,
    )
    .await;

    let call = || async {
        client
            .gql::<Value>("query { post { author } }")
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

#[ignore]
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
    env.write_resolver(
        "greet.js",
        r"
        export default function Resolver(parent, args, context, info) {
            let rng = (Math.random() + 1).toString(36).substring(7)
            return `Hello ${args.name}! ${rng}`
        }
    ",
    );
    let client = start_grafbase(
        &mut env,
        r#"
            extend schema @cache(rules: [{maxAge: 10, types: "Query"}])

            type Query {
                greet(name: String): String! @resolver(name: "greet")
            }
        "#,
    )
    .await;

    let call = |variables: serde_json::Value| async {
        let response = client
            .gql::<Value>("query Greeting($name: String!) { greet(name: $name) }")
            .variables(variables)
            .into_reqwest_builder()
            .send()
            .await
            .unwrap();
        (
            response.headers().clone(),
            response.json::<serde_json::Value>().await.unwrap(),
        )
    };

    let (headers, hello_content) = call(serde_json::json!({ "name": "hello" })).await;
    println!("{}", serde_json::to_string_pretty(&hello_content).unwrap());
    assert_eq!(
        headers.get(GRAFBASE_CACHE_HEADER).map(|v| v.to_str().unwrap()),
        Some("MISS")
    );
    assert_eq!(
        headers.typed_get::<CacheControl>(),
        Some(CacheControl::new().with_public().with_max_age(Duration::from_secs(10)))
    );

    let (headers, world_content) = call(serde_json::json!({ "name": "world" })).await;
    assert_eq!(
        headers.get(GRAFBASE_CACHE_HEADER).map(|v| v.to_str().unwrap()),
        Some("MISS")
    );
    assert_eq!(
        headers.typed_get::<CacheControl>(),
        Some(CacheControl::new().with_public().with_max_age(Duration::from_secs(10)))
    );

    let (headers, content) = call(serde_json::json!({ "name": "hello" })).await;
    assert_eq!(content, hello_content);
    assert_eq!(
        headers.get(GRAFBASE_CACHE_HEADER).map(|v| v.to_str().unwrap()),
        Some("HIT")
    );
    assert_eq!(headers.typed_get::<CacheControl>(), None);

    let (headers, content) = call(serde_json::json!({ "name": "world" })).await;
    assert_eq!(content, world_content);
    assert_eq!(
        headers.get(GRAFBASE_CACHE_HEADER).map(|v| v.to_str().unwrap()),
        Some("HIT")
    );
    assert_eq!(headers.typed_get::<CacheControl>(), None);
}

#[tokio::test(flavor = "multi_thread")]
async fn no_cache_header_when_caching_is_not_used() {
    let mut env = Environment::init_async().await;
    env.write_resolver(
        "title.js",
        r"
        export default function Resolver(parent, args, context, info) {
            return 'Hello!'
        }
    ",
    );
    let client = start_grafbase(
        &mut env,
        r#"
            type Query {
                title: String! @resolver(name: "title")
            }
        "#,
    )
    .await;

    let response = client
        .gql::<Value>("query { title }")
        .into_reqwest_builder()
        .send()
        .await
        .unwrap();
    assert_eq!(header(&response, GRAFBASE_CACHE_HEADER), None);
    assert_eq!(response.headers().typed_get::<CacheControl>(), None);
}

async fn start_grafbase(env: &mut Environment, schema: impl AsRef<str> + Display) -> AsyncClient {
    env.grafbase_init(GraphType::Single);
    env.write_schema(schema);
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    client
}
