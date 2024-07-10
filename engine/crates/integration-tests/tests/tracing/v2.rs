use tracing::Level;
use tracing_mock::{expect, subscriber};

use engine_v2::Engine;
use grafbase_tracing::span::gql::GRAPHQL_SPAN_NAME;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{federation::GatewayV2Ext, runtime};

#[test]
fn query_bad_request() {
    runtime().block_on(async {
        // prepare
        let span = expect::span().at_level(Level::INFO).named(GRAPHQL_SPAN_NAME);

        let (subscriber, handle) = subscriber::mock()
            .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
            .enter(span.clone())
            .record(
                span.clone(),
                expect::field("gql.operation.name").with_value(&"__type_name"),
            )
            .record(span.clone(), expect::field("otel.name").with_value(&"__type_name"))
            .record(
                span.clone(),
                expect::field("gql.operation.query").with_value(&"{\n  __type_name\n}\n"),
            )
            .record(span.clone(), expect::field("gql.operation.type").with_value(&"query"))
            .record(
                span.clone(),
                expect::field("gql.response.status").with_value(&"REQUEST_ERROR"),
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
        let _ = engine.execute("{ __type_name }").await;

        // assert
        handle.assert_finished();
    })
}
