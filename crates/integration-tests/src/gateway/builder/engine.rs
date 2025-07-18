use std::{path::Path, str::FromStr, sync::Arc};

use crate::gateway::{TestRuntimeBuilder, subgraph::Subgraphs};
use extension_catalog::ExtensionCatalog;
use gateway_config::Config;

use engine::ContractAwareEngine;

use super::{TestConfig, TestRuntime};

pub(super) async fn build(
    tmpdir: &Path,
    federated_sdl: Option<String>,
    mut config: TestConfig,
    runtime: TestRuntimeBuilder,
    subgraphs: &Subgraphs,
) -> Result<(Arc<ContractAwareEngine<TestRuntime>>, ExtensionCatalog), String> {
    let federated_sdl = {
        let mut federated_graph = match federated_sdl {
            Some(sdl) => graphql_composition::FederatedGraph::from_sdl(&sdl).unwrap(),
            None => {
                if !subgraphs.is_empty() {
                    let extensions = runtime.extensions.iter_with_url().collect::<Vec<_>>();
                    let mut subgraphs =
                        subgraphs
                            .iter()
                            .fold(graphql_composition::Subgraphs::default(), |mut subgraphs, subgraph| {
                                let url = subgraph.url();

                                // Quite ugly to replace directly, but should work most of time considering we append
                                // the version number
                                let sdl = extensions.iter().fold(subgraph.sdl(), |sdl, (manifest, url)| {
                                    sdl.replace(&manifest.id.to_string(), url.as_str()).into()
                                });

                                subgraphs
                                    .ingest_str(&sdl, subgraph.name(), url.as_ref().map(url::Url::as_str))
                                    .expect("schema to be well formed");
                                subgraphs
                            });

                    subgraphs.ingest_loaded_extensions(extensions.into_iter().map(|(manifest, url)| {
                        graphql_composition::LoadedExtension::new(url.to_string(), manifest.name().to_string())
                    }));

                    graphql_composition::compose(&subgraphs)
                        .warnings_are_fatal()
                        .into_result()
                        .expect("schemas to compose succesfully")
                } else {
                    graphql_composition::FederatedGraph::default()
                }
            }
        };

        for extension in &mut federated_graph.extensions {
            if url::Url::from_str(&federated_graph.strings[usize::from(extension.url)]).is_ok() {
                continue;
            }
            let url = runtime
                .extensions
                .get_url(&federated_graph.strings[usize::from(extension.url)]);
            extension.url = federated_graph.strings.len().into();
            federated_graph.strings.push(url.to_string());
        }

        // Ensure SDL/JSON serialization work as a expected
        let sdl = graphql_composition::render_federated_sdl(&federated_graph).expect("render_federated_sdl()");
        println!("=== SDL ===\n{sdl}\n");
        sdl
    };

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

    let config_path = tmpdir.join("grafbase.toml");
    std::fs::write(tmpdir.join("grafbase.toml"), &config.toml).unwrap();
    let mut config = Config::load(config_path).unwrap().unwrap();

    let schema = Arc::new(
        engine::Schema::builder(&federated_sdl)
            .config(&config)
            .extensions(Some(tmpdir), runtime.extensions.catalog())
            .build()
            .await
            .map_err(|err| err.to_string())?,
    );

    let (runtime, extension_catalog) = runtime.finalize_runtime_and_config(&mut config, &schema).await?;

    println!("=== CONFIG ===\n{config:#?}\n");

    let engine = engine::ContractAwareEngine::new(schema, runtime);

    Ok((Arc::new(engine), extension_catalog))
}
