use std::sync::Arc;

use axum::Router;
use engine::ContractAwareEngine;
use federated_server::router::RouterConfig;
use gateway_config::Config;

use super::TestRuntime;

pub(super) async fn build(engine: Arc<ContractAwareEngine<TestRuntime>>, config: Config) -> Router {
    let (_, engine_watcher) = tokio::sync::watch::channel(engine.clone());

    let router_config = RouterConfig {
        config,
        engine: engine_watcher,
        server_runtime: (),
        extensions: engine.no_contract.runtime.gateway_extensions.clone(),
        inject_layers_before_cors: |r| r,
    };

    let (router, _) = federated_server::router::create(router_config).await.unwrap();

    router
}
