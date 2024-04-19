use tracing::Level;
use tracing_mock::{expect, subscriber};

use engine_v2::Engine;
use grafbase_tracing::span::{gql::GRAPHQL_SPAN_NAME, subgraph::SUBGRAPH_SPAN_NAME};
use graphql_mocks::{FakeFederationProductsSchema, FakeGithubSchema, MockGraphQlServer};
use integration_tests::{federation::GatewayV2Ext, runtime};

#[test]
fn query_bad_request() {
    runtime().block_on(async {
        // prepare
        let query = "";
        let span = expect::span().at_level(Level::INFO).named(GRAPHQL_SPAN_NAME);

        let (subscriber, handle) = subscriber::mock()
            .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
            .enter(span.clone())
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
    })
}

#[test]
fn query_named() {
    runtime().block_on(async {
        // prepare
        let query = "query Named { serverVersion }";
        let graphql_span = expect::span().at_level(Level::INFO).named(GRAPHQL_SPAN_NAME);
        let subgraphql_span = expect::span().at_level(Level::INFO).named(SUBGRAPH_SPAN_NAME);

        let (subscriber, handle) = subscriber::mock()
            .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
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
                subgraphql_span
                    .clone()
                    .with_field(expect::field("subgraph.name").with_value(&"github"))
                    .with_field(expect::field("subgraph.gql.document").with_value(&"query {\n  serverVersion\n}"))
                    .with_field(expect::field("subgraph.gql.operation.type").with_value(&"query")),
            )
            .enter(subgraphql_span.clone())
            .exit(subgraphql_span.clone())
            .exit(graphql_span.clone())
            .run_with_handle();

        let _default = tracing::subscriber::set_default(subscriber);

        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("github", &github_mock)
            .await
            .finish()
            .await;

        // act
        let _ = engine.execute(query).await;

        // assert
        handle.assert_finished();
    })
}

#[test]
fn subscription() {
    runtime().block_on(async {
        // prepare
        let query = r"
                subscription Sub {
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
            .enter(span.clone())
            .record(span.clone(), expect::field("gql.operation.name").with_value(&"Sub"))
            .record(
                span.clone(),
                expect::field("gql.operation.type").with_value(&"subscription"),
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

        let _ = engine.execute(query).into_multipart_stream().collect::<Vec<_>>().await;

        // assert
        handle.assert_finished();
    })
}
