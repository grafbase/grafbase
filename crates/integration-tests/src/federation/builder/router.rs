use std::sync::Arc;

use axum::Router;
use engine::Engine;
use gateway_config::Config;

use super::TestRuntime;

pub(super) async fn build(engine: Arc<Engine<TestRuntime>>, config: Config) -> Router {
    let (_, engine_watcher) = tokio::sync::watch::channel(engine.clone());
    federated_server::router(config, engine_watcher, (), |r| r)
        .await
        .unwrap()
}
