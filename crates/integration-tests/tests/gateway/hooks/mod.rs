mod authorize_edge_node_post_execution;
mod authorize_edge_pre_execution;
mod authorize_node_pre_execution;
mod authorize_parent_edge_post_execution;
mod on_gateway_request;
mod on_subgraph_request;

use futures::Future;
use graphql_mocks::SecureSchema;
use integration_tests::{
    gateway::{DynamicHooks, Gateway},
    runtime,
};

fn with_engine_for_auth<F, O>(hooks: impl Into<DynamicHooks>, f: impl FnOnce(Gateway) -> F) -> O
where
    F: Future<Output = O>,
{
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(SecureSchema)
            .with_mock_hooks(hooks)
            .build()
            .await;

        f(engine).await
    })
}
