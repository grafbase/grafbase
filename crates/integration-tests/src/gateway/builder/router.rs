use std::sync::Arc;

use axum::Router;
use engine::ContractAwareEngine;
use extension_catalog::ExtensionCatalog;
use federated_server::router::RouterConfig;
use gateway_config::Config;

use super::TestRuntime;

pub(super) async fn build(
    engine: Arc<ContractAwareEngine<TestRuntime>>,
    config: Config,
    extension_catalog: ExtensionCatalog,
) -> Router {
    let (_, engine_watcher) = tokio::sync::watch::channel(engine.clone());

    let router_config = RouterConfig {
        config,
        engine: engine_watcher,
        server_runtime: (),
        extension_catalog,
        extensions: engine.no_contract.runtime.gateway_extensions.clone(),
        listen_address: None,
    };

    let (router, _) = federated_server::router::create(router_config).await.unwrap();

    router
}
