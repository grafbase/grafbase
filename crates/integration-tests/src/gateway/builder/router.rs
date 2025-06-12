use std::sync::Arc;

use axum::Router;
use engine::Engine;
use extension_catalog::Extension;
use gateway_config::Config;

use super::TestRuntime;

pub(super) async fn build(
    engine: Arc<Engine<TestRuntime>>,
    config: Config,
    hooks_extension: Option<Extension>,
) -> Router {
    let (_, engine_watcher) = tokio::sync::watch::channel(engine.clone());

    let (router, _) = federated_server::router(config, engine_watcher, (), hooks_extension, |r| r)
        .await
        .unwrap();

    router
}
