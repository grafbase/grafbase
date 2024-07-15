mod authorize_edge_pre_execution;
mod authorize_node_pre_execution;
mod on_gateway_request;

use engine_v2::Engine;
use futures::Future;
use graphql_mocks::{MockGraphQlServer, SecureSchema};
use integration_tests::{
    federation::{EngineV2Ext, TestEngineV2},
    runtime,
};
use runtime::hooks::DynamicHooks;

fn with_engine_for_auth<F, O>(hooks: impl Into<DynamicHooks>, f: impl FnOnce(TestEngineV2) -> F) -> O
where
    F: Future<Output = O>,
{
    runtime().block_on(async move {
        let secure_mock = MockGraphQlServer::new(SecureSchema::default()).await;

        let engine = Engine::builder()
            .with_subgraph("secure", &secure_mock)
            .with_hooks(hooks)
            .build()
            .await;

        f(engine).await
    })
}
