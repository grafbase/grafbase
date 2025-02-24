use std::{str::FromStr, sync::Arc};

use crate::federation::{TestRuntimeContext, subgraph::Subgraphs};
use federated_graph::FederatedGraph;
use grafbase_telemetry::metrics::meter_from_global_provider;
use runtime::hooks::DynamicHooks;
use runtime_local::wasi::hooks::{self, AccessLogSender, ComponentLoader, HooksWasi};

use engine::Engine;

use super::{TestConfig, TestRuntime};

pub(super) async fn build(
    federated_sdl: Option<String>,
    mut config: TestConfig,
    mut runtime: TestRuntime,
    subgraphs: &Subgraphs,
) -> Result<(Arc<Engine<TestRuntime>>, TestRuntimeContext), String> {
    let federated_graph = {
        let mut federated_graph = match federated_sdl {
            Some(sdl) => federated_graph::FederatedGraph::from_sdl(&sdl).unwrap(),
            None => {
                if !subgraphs.is_empty() {
                    let extensions = runtime.extensions.iter().collect::<Vec<_>>();
                    let mut subgraphs =
                        subgraphs
                            .iter()
                            .fold(graphql_composition::Subgraphs::default(), |mut subgraphs, subgraph| {
                                let url = subgraph.url();

                                // Quite ugly to replace directly, but should work most of time considering we append
                                // the version number
                                let sdl = extensions.iter().fold(subgraph.sdl(), |sdl, (url, manifest)| {
                                    sdl.replace(&manifest.id.to_string(), url.as_str()).into()
                                });

                                subgraphs
                                    .ingest_str(&sdl, subgraph.name(), url.as_ref().map(url::Url::as_str))
                                    .expect("schema to be well formed");
                                subgraphs
                            });
                    subgraphs.ingest_loaded_extensions(extensions.into_iter().map(|(url, manifest)| {
                        graphql_composition::LoadedExtension::new(url.to_string(), manifest.name().to_string())
                    }));
                    graphql_composition::compose(&subgraphs)
                        .into_result()
                        .expect("schemas to compose succesfully")
                } else {
                    federated_graph::FederatedGraph::default()
                }
            }
        };

        for extension in &mut federated_graph.extensions {
            if url::Url::from_str(&federated_graph.strings[usize::from(extension.url)]).is_ok() {
                continue;
            }
            let url = url::Url::from_file_path(
                runtime
                    .extensions
                    .tmpdir
                    .path()
                    .join(&federated_graph.strings[usize::from(extension.url)]),
            )
            .unwrap();
            extension.url = federated_graph.strings.len().into();
            federated_graph.strings.push(url.to_string());
        }

        // Ensure SDL/JSON serialization work as a expected
        let sdl = federated_graph::render_federated_sdl(&federated_graph).expect("render_federated_sdl()");
        println!("{sdl}");
        FederatedGraph::from_sdl(&sdl).unwrap()
    };

    let counter = grafbase_telemetry::metrics::meter_from_global_provider()
        .i64_up_down_counter("grafbase.gateway.access_log.pending")
        .build();

    let (access_log_sender, access_log_receiver) = hooks::create_access_log_channel(false, counter);

    if config.add_websocket_url {
        for subgraph in subgraphs.iter() {
            let name = subgraph.name();
            if let Some(websocket_url) = subgraph.websocket_url() {
                config.toml.push_str(&indoc::formatdoc! {r#"
                    [subgraphs.{name}]
                    websocket_url = "{websocket_url}"
                "#});
            }
        }
    }

    let config = toml::from_str(&config.toml).unwrap();

    update_runtime_with_toml_config(&mut runtime, &config, access_log_sender).await;

    let schema = engine::Schema::build(
        &config,
        &federated_graph,
        runtime.extensions.catalog(),
        engine::SchemaVersion::from(ulid::Ulid::new().to_bytes()),
    )
    .await
    .map_err(|err| err.to_string())?;
    let engine = engine::Engine::new(Arc::new(schema), runtime).await;
    let ctx = TestRuntimeContext { access_log_receiver };

    Ok((Arc::new(engine), ctx))
}

async fn update_runtime_with_toml_config(
    runtime: &mut TestRuntime,
    config: &gateway_config::Config,
    access_log_sender: AccessLogSender,
) {
    if let Some(hooks_config) = config.hooks.clone() {
        let loader = ComponentLoader::hooks(hooks_config)
            .ok()
            .flatten()
            .expect("Wasm examples weren't built, please run:\ncd crates/wasi-component-loader/examples && cargo build --target wasm32-wasip2");

        let meter = meter_from_global_provider();
        let hooks = HooksWasi::new(Some(loader), None, &meter, access_log_sender).await;

        runtime.hooks = DynamicHooks::wrap(hooks);
    }
}
