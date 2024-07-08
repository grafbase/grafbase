mod authorize_edge_pre_execution;
mod authorize_node_pre_execution;
mod on_gateway_request;

use engine_v2::Engine;
use futures::Future;
use graphql_mocks::{MockGraphQlServer, SecureSchema};
use integration_tests::{
    federation::{GatewayV2Ext, TestFederationEngine},
    runtime,
};
use runtime::hooks::DynamicHooks;

fn with_engine_for_auth<F, O>(hooks: impl Into<DynamicHooks>, f: impl FnOnce(TestFederationEngine) -> F) -> O
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
