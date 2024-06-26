use engine_v2::Engine;
use futures::Future;
use graphql_mocks::{MockGraphQlServer, SecureSchema};
use integration_tests::{
    federation::{GatewayV2Ext, TestFederationEngine, TestHooks},
    runtime,
};

mod rule;

fn with_prepared_engine<F, O>(hooks: TestHooks, f: impl FnOnce(TestFederationEngine) -> F) -> O
where
    F: Future<Output = O>,
{
    runtime().block_on(async move {
        let secure_mock = MockGraphQlServer::new(SecureSchema::default()).await;

        let engine = Engine::builder()
            .with_schema("secure", &secure_mock)
            .await
            .with_hooks(hooks)
            .finish()
            .await;

        f(engine).await
    })
}
