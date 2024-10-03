use std::{collections::HashMap, sync::Arc};

use crate::federation::{subgraph::Subgraphs, TestRuntimeContext};
use engine_config_builder::{build_with_sdl_config, build_with_toml_config};
use federated_graph::FederatedGraphV3;
use grafbase_telemetry::metrics::meter_from_global_provider;
use graphql_composition::VersionedFederatedGraph;
use parser_sdl::{connector_parsers::MockConnectorParsers, federation::FederatedGraphConfig};
use runtime::hooks::DynamicHooks;
use runtime_local::{
    hooks::{self, ChannelLogSender},
    ComponentLoader, HooksWasi,
};

use engine_v2::Engine;

use super::{ConfigSource, TestRuntime};

pub(super) async fn build(
    federated_sdl: Option<String>,
    config_source: Option<ConfigSource>,
    mut runtime: TestRuntime,
    subgraphs: &Subgraphs,
) -> (Arc<Engine<TestRuntime>>, TestRuntimeContext) {
    let graph = federated_sdl
        .map(|sdl| VersionedFederatedGraph::from_sdl(&sdl).unwrap())
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
                VersionedFederatedGraph::V3(FederatedGraphV3::default())
            }
        });

    // Ensure SDL/JSON serialization work as a expected
    let graph = {
        let sdl = graph.into_federated_sdl();
        println!("{sdl}");
        let mut graph = VersionedFederatedGraph::from_sdl(&sdl).unwrap();
        let json = serde_json::to_value(&graph).unwrap();
        graph = serde_json::from_value(json).unwrap();
        graph
    };

    let counter = grafbase_telemetry::metrics::meter_from_global_provider()
        .i64_up_down_counter("grafbase.gateway.access_log.pending")
        .init();

    let (access_log_sender, access_log_receiver) = hooks::create_log_channel(false, counter);

    let config = match config_source {
        Some(ConfigSource::Toml(toml)) => {
            let config: gateway_config::Config = toml::from_str(&toml).unwrap();

            update_runtime_with_toml_config(&mut runtime, &config, access_log_sender);
            build_with_toml_config(&config, graph.into_latest())
        }
        Some(ConfigSource::Sdl(mut sdl)) => {
            sdl.push_str("\nextend schema @graph(type: federated)");
            let config = parse_sdl_config(&sdl).await;
            build_with_sdl_config(&config, graph.into_latest())
        }
        Some(ConfigSource::SdlWebsocket) => {
            let mut sdl = String::new();
            sdl.push_str("\nextend schema @graph(type: federated)");
            for subgraph in subgraphs.iter() {
                sdl.push_str(&format!(
                    "\nextend schema @subgraph(name: \"{}\", websocketUrl: \"{}\")",
                    subgraph.name(),
                    subgraph.websocket_url()
                ));
            }

            let config = parse_sdl_config(&sdl).await;
            build_with_sdl_config(&config, graph.into_latest())
        }
        None => build_with_sdl_config(&Default::default(), graph.into_latest()),
    }
    .into_latest();

    let schema =
        engine_v2::Schema::build(config, engine_v2::SchemaVersion::from(ulid::Ulid::new().to_bytes())).unwrap();
    let engine = engine_v2::Engine::new(Arc::new(schema), runtime).await;
    let ctx = TestRuntimeContext { access_log_receiver };

    (Arc::new(engine), ctx)
}

fn update_runtime_with_toml_config(
    runtime: &mut TestRuntime,
    config: &gateway_config::Config,
    access_log_sender: ChannelLogSender,
) {
    if let Some(hooks_config) = config.hooks.clone() {
        let loader = ComponentLoader::new(hooks_config)
            .ok()
            .flatten()
            .expect("Wasm examples weren't built, please run:\ncd engine/crates/wasi-component-loader/examples && cargo component build");

        let meter = meter_from_global_provider();
        runtime.hooks = DynamicHooks::wrap(HooksWasi::new(Some(loader), &meter, access_log_sender));
    }
}

async fn parse_sdl_config(sdl: &str) -> FederatedGraphConfig {
    parser_sdl::parse(sdl, &HashMap::new(), &MockConnectorParsers::default())
        .await
        .expect("supergraph config SDL to be valid")
        .federated_graph_config
        .unwrap_or_default()
}
