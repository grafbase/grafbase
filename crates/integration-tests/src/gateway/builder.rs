mod bench;

use std::{any::TypeId, collections::HashSet, fmt::Display, path::PathBuf, str::FromStr as _, sync::Arc};

use crate::{TestTrustedDocument, mock_trusted_documents::MockTrustedDocumentsClient};
pub use bench::*;
use federated_server::router::RouterConfig;
use futures::{FutureExt, future::BoxFuture};
use gateway_config::Config;
use graphql_mocks::MockGraphQlServer;
use runtime::{
    fetch::dynamic::DynamicFetcher,
    trusted_documents_client::{self, TrustedDocumentsEnforcementMode},
};
use tempfile::TempDir;

use super::{
    AnyExtension, DockerSubgraph, ExtensionsBuilder, Gateway, TestRuntime, TestRuntimeBuilder,
    subgraph::{Subgraph, Subgraphs},
};

#[derive(Default)]
struct TestConfig {
    toml: String,
    add_websocket_url: bool,
}

#[must_use]
pub struct GatewayBuilder {
    tmpdir: TempDir,
    federated_sdl: Option<String>,
    mock_subgraphs: Vec<(TypeId, String, BoxFuture<'static, MockGraphQlServer>)>,
    docker_subgraphs: HashSet<DockerSubgraph>,
    virtual_subgraphs: Vec<(String, String)>,
    config: TestConfig,
    runtime: TestRuntimeBuilder,
}

impl Default for GatewayBuilder {
    fn default() -> Self {
        let tmpdir = tempfile::tempdir().unwrap();
        let extensions_dir = tmpdir.path().join("extensions");
        Self {
            federated_sdl: Default::default(),
            mock_subgraphs: Default::default(),
            docker_subgraphs: Default::default(),
            virtual_subgraphs: Default::default(),
            config: Default::default(),
            runtime: TestRuntimeBuilder {
                trusted_documents: Default::default(),
                fetcher: Default::default(),
                extensions: ExtensionsBuilder::new(extensions_dir),
            },
            tmpdir,
        }
    }
}

impl GatewayBuilder {
    pub fn with_toml_config(mut self, toml: impl Display) -> Self {
        assert!(self.config.toml.is_empty(), "overwriting config!");
        self.config.toml = toml.to_string();
        self
    }

    pub fn with_websocket_urls(mut self) -> Self {
        self.config.add_websocket_url = true;
        self
    }

    pub fn with_subgraph<S: graphql_mocks::Subgraph>(mut self, subgraph: S) -> Self {
        let name = subgraph.name();
        self.mock_subgraphs
            .push((std::any::TypeId::of::<S>(), name.to_string(), subgraph.start().boxed()));
        self
    }

    pub fn with_docker_subgraph(mut self, subgraph: DockerSubgraph) -> Self {
        self.docker_subgraphs.insert(subgraph);
        self
    }

    pub fn with_subgraph_sdl(mut self, name: &str, sdl: impl Display) -> Self {
        self.virtual_subgraphs.push((name.to_string(), sdl.to_string()));
        self
    }

    /// Will bypass the composition of subgraphs and be used at its stead.
    pub fn with_federated_sdl(mut self, sdl: impl Display) -> Self {
        self.federated_sdl = Some(sdl.to_string());
        self
    }

    //-- Runtime customization --
    // Prefer passing through either the TOML / SDL config when relevant, see update_runtime_with_toml_config
    //--

    pub fn with_mock_trusted_documents(
        mut self,
        enforcement_mode: TrustedDocumentsEnforcementMode,
        documents: Vec<TestTrustedDocument>,
    ) -> Self {
        self.runtime.trusted_documents = Some(trusted_documents_client::Client::new(MockTrustedDocumentsClient {
            documents,
            enforcement_mode,
        }));
        self
    }

    pub fn with_extension(mut self, ext: impl AnyExtension) -> Self {
        ext.register(&mut self.runtime.extensions);
        self
    }

    pub fn with_mock_fetcher(mut self, fetcher: impl Into<DynamicFetcher>) -> Self {
        self.runtime.fetcher = Some(fetcher.into());
        self
    }
    //-- Runtime customization --

    pub async fn build(self) -> Gateway {
        self.try_build().await.unwrap()
    }

    pub async fn try_build(self) -> anyhow::Result<Gateway> {
        let Self {
            tmpdir,
            federated_sdl,
            mock_subgraphs,
            docker_subgraphs,
            virtual_subgraphs,
            mut config,
            runtime,
        } = self;

        let subgraphs = Subgraphs::load(
            mock_subgraphs,
            docker_subgraphs,
            virtual_subgraphs
                .into_iter()
                .map(|(name, sdl)| Subgraph::Virtual { name, sdl })
                .collect(),
        )
        .await;

        let federated_sdl = {
            let mut federated_graph = match federated_sdl {
                Some(sdl) => graphql_composition::FederatedGraph::from_sdl(&sdl).unwrap(),
                None => {
                    if !subgraphs.is_empty() {
                        let mut composed_subgraphs = graphql_composition::Subgraphs::default();

                        composed_subgraphs.ingest_loaded_extensions(runtime.extensions.iter_with_url().map(
                            |(manifest, url)| graphql_composition::LoadedExtension {
                                link_url: manifest.id.to_string(),
                                url,
                                name: manifest.id.name.to_string(),
                            },
                        ));

                        for subgraph in subgraphs.iter() {
                            composed_subgraphs.ingest_str(
                                subgraph.sdl().as_ref(),
                                subgraph.name(),
                                subgraph.url().as_ref().map(url::Url::as_str),
                            )?;
                        }

                        graphql_composition::compose(&mut composed_subgraphs)
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

        let config = {
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

            let config_path = tmpdir.path().join("grafbase.toml");
            std::fs::write(tmpdir.path().join("grafbase.toml"), &config.toml).unwrap();
            let mut config = Config::load(config_path).map_err(|err| anyhow::anyhow!(err))?.unwrap();
            if config.wasm.is_none() {
                let crate_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
                let project_root = crate_path.parent().unwrap().parent().unwrap();
                let cache_path = project_root.join(".grafbase").join("wasm-cache");

                config.wasm = Some(gateway_config::WasmConfig {
                    cache_path: Some(cache_path),
                });
            }
            
            // Update config with extension settings before creating the schema
            runtime.extensions.update_config(&mut config)?;
            
            Arc::new(config)
        };

        let schema = Arc::new(
            ::engine::Schema::builder(&federated_sdl)
                .config(config.clone())
                .extensions(runtime.extensions.catalog())
                .build()
                .await
                .map_err(|err| anyhow::anyhow!(err))?,
        );

        let (runtime, extension_catalog) = runtime.finalize_runtime(&config, &schema).await?;

        let engine = Arc::new(::engine::ContractAwareEngine::new(schema, runtime));

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

        Ok(Gateway {
            tmpdir: Arc::new(tmpdir),
            router,
            engine,
            subgraphs,
        })
    }
}
