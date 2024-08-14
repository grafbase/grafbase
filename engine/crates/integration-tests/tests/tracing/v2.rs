use tracing::Level;
use tracing_mock::{expect, subscriber};

use engine_v2::Engine;
use grafbase_telemetry::span::gql::GRAPHQL_SPAN_NAME;
use graphql_mocks::FakeGithubSchema;
use integration_tests::{federation::EngineV2Ext, runtime};

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

        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        // act
        let _ = engine.post("{ __type_name }").await;

        // assert
        handle.assert_finished();
    })
}
