use std::sync::Arc;

use axum::Router;
use engine::Engine;
use extension_catalog::Extension;
use gateway_config::Config;
use grafbase_telemetry::otel::opentelemetry::global;
use wasi_component_loader::create_access_log_channel;

use super::TestRuntime;

pub(super) async fn build(
    engine: Arc<Engine<TestRuntime>>,
    config: Config,
    hooks_extension: Option<Extension>,
) -> Router {
    let (_, engine_watcher) = tokio::sync::watch::channel(engine.clone());
    let (log_sender, _) = create_access_log_channel(true, global::meter("foo").i64_up_down_counter("bar").build());

    let (router, _) = federated_server::router(config, engine_watcher, (), hooks_extension, log_sender, |r| r)
        .await
        .unwrap();

    router
}
