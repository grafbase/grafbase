use std::sync::Arc;

use axum::Router;
use engine::Engine;
use gateway_config::Config;
use wasi_component_loader::extension::WasmHooks;

use super::TestRuntime;

pub(super) async fn build(engine: Arc<Engine<TestRuntime>>, config: Config, hooks: Option<WasmHooks>) -> Router {
    let (_, engine_watcher) = tokio::sync::watch::channel(engine.clone());

    let (router, _) = match hooks {
        Some(hooks) => federated_server::router(config, engine_watcher, (), hooks, |r| r)
            .await
            .unwrap(),
        None => federated_server::router(config, engine_watcher, (), (), |r| r)
            .await
            .unwrap(),
    };

    router
}
