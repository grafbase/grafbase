use std::sync::Arc;

use crate::federation::{subgraph::Subgraphs, TestRuntimeContext};
use engine_config_builder::build_with_toml_config;
use federated_graph::FederatedGraph;
use grafbase_telemetry::metrics::meter_from_global_provider;
use runtime::hooks::DynamicHooks;
use runtime_local::{
    hooks::{self, ChannelLogSender},
    ComponentLoader, HooksWasi,
};

use engine::Engine;

use super::{ConfigSource, TestRuntime};

pub(super) async fn build(
    federated_sdl: Option<String>,
    config: Option<ConfigSource>,
    mut runtime: TestRuntime,
    subgraphs: &Subgraphs,
) -> (Arc<Engine<TestRuntime>>, TestRuntimeContext) {
    let graph = federated_sdl
        .map(|sdl| federated_graph::FederatedGraph::from_sdl(&sdl).unwrap())
        .unwrap_or_else(|| {
            if !subgraphs.is_empty() {
                graphql_composition::compose(&subgraphs.iter().fold(
                    graphql_composition::Subgraphs::default(),
                    |mut subgraphs, subgraph| {
                        subgraphs
                            .ingest_str(subgraph.sdl().as_ref(), subgraph.name(), subgraph.url().as_ref())
                            .expect("schema to be well formed");
                        subgraphs
                    },
                ))
                .into_result()
                .expect("schemas to compose succesfully")
            } else {
                federated_graph::FederatedGraph::default()
            }
        });

    // Ensure SDL/JSON serialization work as a expected
    let graph = {
        let sdl = federated_graph::render_federated_sdl(&graph).expect("render_federated_sdl()");
        println!("{sdl}");
        FederatedGraph::from_sdl(&sdl).unwrap()
    };

    let counter = grafbase_telemetry::metrics::meter_from_global_provider()
        .i64_up_down_counter("grafbase.gateway.access_log.pending")
        .build();

    let (access_log_sender, access_log_receiver) = hooks::create_log_channel(false, counter);

    let config = match config {
        Some(ConfigSource::Toml(ref config_toml)) => toml::from_str(config_toml).unwrap(),
        Some(ConfigSource::TomlWebsocket(mut config_toml)) => {
            for subgraph in subgraphs.iter() {
                let name = subgraph.name();
                let websocket_url = subgraph.websocket_url();

                config_toml.push_str(&indoc::formatdoc! {r#"
                    [subgraphs.{name}]
                    websocket_url = "{websocket_url}"
                "#});
            }

            toml::from_str(&config_toml).unwrap()
        }
        None => gateway_config::Config::default(),
    };

    update_runtime_with_toml_config(&mut runtime, &config, access_log_sender).await;

    let config = build_with_toml_config(&config, graph);

    let schema = engine::Schema::build(config, engine::SchemaVersion::from(ulid::Ulid::new().to_bytes())).unwrap();
    let engine = engine::Engine::new(Arc::new(schema), runtime).await;
    let ctx = TestRuntimeContext { access_log_receiver };

    (Arc::new(engine), ctx)
}

async fn update_runtime_with_toml_config(
    runtime: &mut TestRuntime,
    config: &gateway_config::Config,
    access_log_sender: ChannelLogSender,
) {
    if let Some(hooks_config) = config.hooks.clone() {
        let loader = ComponentLoader::new(hooks_config)
            .ok()
            .flatten()
            .expect("Wasm examples weren't built, please run:\ncd crates/wasi-component-loader/examples && cargo build --target wasm32-wasip2");

        let meter = meter_from_global_provider();
        let hooks = HooksWasi::new(Some(loader), None, &meter, access_log_sender).await;

        runtime.hooks = DynamicHooks::wrap(hooks);
    }
}
