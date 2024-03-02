use tracing::Level;
use tracing_mock::{expect, subscriber};

use engine_v2::Engine;
use grafbase_tracing::span::gql::SPAN_NAME as GRAPHQL_SPAN_NAME;
use graphql_mocks::{FakeFederationProductsSchema, FakeGithubSchema, MockGraphQlServer};
use integration_tests::federation::EngineV2Ext;

#[tokio::test(flavor = "current_thread")]
async fn query_bad_request() {
    // prepare
    let query = "";
    let span = expect::span().at_level(Level::INFO).named(GRAPHQL_SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        .new_span(
            span.clone()
                .with_field(expect::field("gql.document").with_value(&query)),
        )
        .record(span.clone(), expect::field("gql.response.has_errors").with_value(&true))
        .run_with_handle();

    let _default = tracing::subscriber::set_default(subscriber);

    let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

    let engine = Engine::builder()
        .with_schema("schema", &github_mock)
        .await
        .finish()
        .await;

    // act
    let _ = engine.execute(query).await;

    // assert
    handle.assert_finished();
}

#[tokio::test(flavor = "current_thread")]
async fn query_named() {
    // prepare
    let query = "query Named { serverVersion }";
    let span = expect::span().at_level(Level::INFO).named(GRAPHQL_SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        .new_span(
            span.clone()
                .with_field(expect::field("gql.document").with_value(&query)),
        )
        .enter(span.clone())
        .record(
            span.clone(),
            expect::field("gql.request.operation.name").with_value(&"Named"),
        )
        .record(
            span.clone(),
            expect::field("gql.request.operation.type").with_value(&"query"),
        )
        .run_with_handle();

    let _default = tracing::subscriber::set_default(subscriber);

    let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

    let engine = Engine::builder()
        .with_schema("schema", &github_mock)
        .await
        .finish()
        .await;

    // act
    let _ = engine.execute(query).await;

    // assert
    handle.assert_finished();
}

#[tokio::test(flavor = "current_thread")]
async fn subscription() {
    // prepare
    let query = r"
                subscription {
                    newProducts {
                        upc
                        name
                        price
                    }
                }
                ";
    let span = expect::span().at_level(Level::INFO).named(GRAPHQL_SPAN_NAME);

    let (subscriber, handle) = subscriber::mock()
        .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        .new_span(
            span.clone()
                .with_field(expect::field("gql.document").with_value(&query)),
        )
        .enter(span.clone())
        .record(
            span.clone(),
            expect::field("gql.request.operation.type").with_value(&"subscription"),
        )
        .run_with_handle();

    let _default = tracing::subscriber::set_default(subscriber);

    // engine
    let products = MockGraphQlServer::new(FakeFederationProductsSchema).await;
    let engine = Engine::builder()
        .with_schema("products", &products)
        .await
        .with_supergraph_config(indoc::formatdoc!(
            r#"
                    extend schema
                      @subgraph(name: "products", websocketUrl: "{}")
                "#,
            products.websocket_url(),
        ))
        .finish()
        .await;

    let _ = engine.execute(query).into_stream().collect::<Vec<_>>().await;

    // assert
    handle.assert_finished();
}
