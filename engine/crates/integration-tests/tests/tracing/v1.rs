use std::sync::Arc;

use serde_json::json;
use tracing::Level;
use tracing_mock::{expect, subscriber};

use engine::{BatchRequest, Registry, Request, StreamingPayload};
use grafbase_tracing::span::gql::GRAPHQL_SPAN_NAME;
use grafbase_tracing::span::resolver::RESOLVER_SPAN_NAME;
use integration_tests::udfs::RustUdfs;
use integration_tests::EngineBuilder;
use runtime::udf::UdfResponse;

#[tokio::test(flavor = "current_thread")]
async fn query_bad_request() {
    // prepare
    let span = expect::span().at_level(Level::INFO).named(GRAPHQL_SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        .new_span(span.clone())
        .enter(span.clone())
        .clone_span(span.clone())
        .record(span.clone(), expect::field("gql.response.has_errors").with_value(&true))
        .drop_span(span.clone())
        .exit(span.clone())
        .enter(span.clone())
        .exit(span.clone())
        .only()
        .run_with_handle();

    let _default = tracing::subscriber::set_default(subscriber);

    // act

    engine::Schema::build(Arc::new(Registry::new()))
        .finish()
        .execute("")
        .await;

    // assert
    handle.assert_finished();
}

#[tokio::test(flavor = "current_thread")]
async fn query() {
    // prepare
    let query = "query { test }";
    let span = expect::span().at_level(Level::INFO).named(GRAPHQL_SPAN_NAME);
    let resolver_span = expect::span().at_level(Level::INFO).named(RESOLVER_SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        .new_span(span.clone())
        .enter(span.clone())
        .record(span.clone(), expect::field("gql.operation.type").with_value(&"query"))
        .new_span(
            resolver_span
                .clone()
                .with_field(expect::field("resolver.name").with_value(&"test")),
        )
        .enter(resolver_span.clone())
        .run_with_handle();

    let _default = tracing::subscriber::set_default(subscriber);

    let schema = r#"
            extend type Query {
                test: String! @resolver(name: "test")
            }
        "#;
    let engine = EngineBuilder::new(schema)
        .with_custom_resolvers(RustUdfs::new().resolver("test", UdfResponse::Success(json!("hello"))))
        .build()
        .await;

    // act
    let _ = engine.execute(query).await;

    // assert
    handle.assert_finished();
}

#[tokio::test(flavor = "current_thread")]
async fn query_named() {
    // prepare
    let query = "query Named { test }";
    let graphql_span = expect::span().at_level(Level::INFO).named(GRAPHQL_SPAN_NAME);
    let resolver_span = expect::span().at_level(Level::INFO).named(RESOLVER_SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        .new_span(graphql_span.clone())
        .enter(graphql_span.clone())
        .record(
            graphql_span.clone(),
            expect::field("gql.operation.name").with_value(&"Named"),
        )
        .record(
            graphql_span.clone(),
            expect::field("gql.operation.type").with_value(&"query"),
        )
        .new_span(
            resolver_span
                .clone()
                .with_field(expect::field("resolver.name").with_value(&"test")),
        )
        .enter(resolver_span.clone())
        .run_with_handle();

    let _default = tracing::subscriber::set_default(subscriber);

    let schema = r#"
            extend type Query {
                test: String! @resolver(name: "test")
            }
        "#;
    let engine = EngineBuilder::new(schema)
        .with_custom_resolvers(RustUdfs::new().resolver("test", UdfResponse::Success(json!("hello"))))
        .build()
        .await;

    // act
    let _ = engine.execute(query).await;

    // assert
    handle.assert_finished();
}

#[tokio::test(flavor = "current_thread")]
async fn batch() {
    // prepare
    let span = expect::span().at_level(Level::INFO).named(GRAPHQL_SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(move |meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        // span #1
        .new_span(span.clone())
        .enter(span.clone())
        .clone_span(span.clone())
        .record(span.clone(), expect::field("gql.response.has_errors").with_value(&true))
        .drop_span(span.clone())
        .exit(span.clone())
        .enter(span.clone())
        .exit(span.clone())
        // span #2
        .new_span(span.clone())
        .enter(span.clone())
        .clone_span(span.clone())
        .record(span.clone(), expect::field("gql.response.has_errors").with_value(&true))
        .drop_span(span.clone())
        .exit(span.clone())
        .enter(span.clone())
        .exit(span.clone())
        .only()
        .run_with_handle();

    let _default = tracing::subscriber::set_default(subscriber);

    // act
    engine::Schema::build(Arc::new(Registry::new()))
        .finish()
        .execute_batch(BatchRequest::Batch(vec![
            Request::new("query-1"),
            Request::new("query-2"),
        ]))
        .await;

    // assert
    handle.assert_finished();
}

#[tokio::test(flavor = "current_thread")]
async fn subscription() {
    use engine::futures_util::StreamExt;

    // prepare
    let span = expect::span().at_level(Level::INFO).named(GRAPHQL_SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        .new_span(span.clone())
        .only()
        .run_with_handle();

    let _default = tracing::subscriber::set_default(subscriber);

    // act
    let _: Vec<StreamingPayload> = engine::Schema::build(Arc::new(Registry::new()))
        .finish()
        .execute_stream("")
        .collect()
        .await;

    // assert
    handle.assert_finished();
}

#[tokio::test(flavor = "current_thread")]
async fn resolvers_with_error() {
    // prepare
    let query = "query { nope }";
    let span = expect::span().at_level(Level::INFO).named(GRAPHQL_SPAN_NAME);
    let resolver_span_error = expect::span().at_level(Level::INFO).named(RESOLVER_SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        .new_span(span.clone())
        .enter(span.clone())
        .record(span.clone(), expect::field("gql.operation.type").with_value(&"query"))
        .new_span(
            resolver_span_error
                .clone()
                .with_field(expect::field("resolver.name").with_value(&"error")),
        )
        .enter(resolver_span_error.clone())
        .exit(resolver_span_error.clone())
        .enter(resolver_span_error.clone())
        .exit(resolver_span_error.clone())
        .record(
            resolver_span_error.clone(),
            expect::field("resolver.invocation.error").with_value(&"Invocation failed"),
        )
        .record(
            resolver_span_error.clone(),
            expect::field("resolver.invocation.is_error").with_value(&true),
        )
        .run_with_handle();

    let _default = tracing::subscriber::set_default(subscriber);

    let schema = r#"
            extend type Query {
                nope: String! @resolver(name: "error")
            }
        "#;
    let engine = EngineBuilder::new(schema)
        .with_custom_resolvers(RustUdfs::new().resolver("error", UdfResponse::Error("nope".to_string())))
        .build()
        .await;

    // act
    let _ = engine.execute(query).await;

    // assert
    handle.assert_finished();
}
