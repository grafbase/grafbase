use serde_json::json;
use tracing::Level;
use tracing_mock::{expect, subscriber};

use engine::{BatchRequest, Registry, Request, StreamingPayload};
use grafbase_tracing::span::gql::SPAN_NAME;
use integration_tests::udfs::RustUdfs;
use integration_tests::EngineBuilder;
use runtime::udf::UdfResponse;

#[tokio::test(flavor = "current_thread")]
async fn query_bad_request() {
    // prepare
    let span = expect::span().at_level(Level::INFO).named(SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        .new_span(span.clone().with_field(expect::field("gql.document").with_value(&"")))
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
    EngineBuilder::new("").build().await.execute("").await;

    // assert
    handle.assert_finished();
}

#[tokio::test(flavor = "current_thread")]
async fn query() {
    // prepare
    let query = "query { test }";
    let span = expect::span().at_level(Level::INFO).named(SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        .new_span(
            span.clone()
                .with_field(expect::field("gql.document").with_value(&query)),
        )
        .enter(span.clone())
        .clone_span(span.clone())
        .record(
            span.clone(),
            expect::field("gql.request.operation.type").with_value(&"query"),
        )
        .drop_span(span.clone())
        .exit(span.clone())
        .enter(span.clone())
        .exit(span.clone())
        .only()
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
    let span = expect::span().at_level(Level::INFO).named(SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        .new_span(
            span.clone()
                .with_field(expect::field("gql.document").with_value(&query)),
        )
        .enter(span.clone())
        .clone_span(span.clone())
        .record(
            span.clone(),
            expect::field("gql.request.operation.name").with_value(&"Named"),
        )
        .record(
            span.clone(),
            expect::field("gql.request.operation.type").with_value(&"query"),
        )
        .drop_span(span.clone())
        .exit(span.clone())
        .enter(span.clone())
        .exit(span.clone())
        .only()
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
    let span = expect::span().at_level(Level::INFO).named(SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(move |meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        // span #1
        .new_span(
            span.clone()
                .with_field(expect::field("gql.document").with_value(&"query-1")),
        )
        .enter(span.clone())
        .clone_span(span.clone())
        .record(span.clone(), expect::field("gql.response.has_errors").with_value(&true))
        .drop_span(span.clone())
        .exit(span.clone())
        .enter(span.clone())
        .exit(span.clone())
        // span #2
        .new_span(
            span.clone()
                .with_field(expect::field("gql.document").with_value(&"query-2")),
        )
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
    engine::Schema::build(Registry::new())
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
    // prepare
    let span = expect::span().at_level(Level::INFO).named(SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        .new_span(span.clone().with_field(expect::field("gql.document").with_value(&"")))
        .only()
        .run_with_handle();

    let _default = tracing::subscriber::set_default(subscriber);

    // act
    let _: Vec<StreamingPayload> = EngineBuilder::new("").build().await.execute_stream("").collect().await;

    // assert
    handle.assert_finished();
}
