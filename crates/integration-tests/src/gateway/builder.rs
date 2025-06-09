mod bench;
mod engine;
mod router;

use std::{any::TypeId, collections::HashSet, fmt::Display, fs, path::PathBuf, sync::Arc};

use crate::{TestTrustedDocument, mock_trusted_documents::MockTrustedDocumentsClient};
pub use bench::*;
use extension_catalog::Extension;
use futures::{FutureExt, future::BoxFuture};
use grafbase_telemetry::otel::opentelemetry::global;
use graphql_mocks::MockGraphQlServer;
use runtime::{
    fetch::dynamic::DynamicFetcher,
    trusted_documents_client::{self, TrustedDocumentsEnforcementMode},
};
use tempfile::TempDir;
use wasi_component_loader::{create_access_log_channel, extension::WasmHooks, resources::SharedResources};

use super::{
    AnyExtension, DockerSubgraph, DynamicHooks, ExtensionsBuilder, Gateway, TestRuntime, TestRuntimeBuilder,
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
                hooks: Default::default(),
                fetcher: Default::default(),
                extensions: ExtensionsBuilder::new(extensions_dir),
                hooks_extension: None,
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

    pub fn with_subgraph_sdl(mut self, name: &str, sdl: &str) -> Self {
        self.virtual_subgraphs.push((name.to_string(), sdl.to_string()));
        self
    }

    /// Will bypass the composition of subgraphs and be used at its stead.
    pub fn with_federated_sdl(mut self, sdl: &str) -> Self {
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

    pub async fn with_hook_extension(mut self, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let (access_log, _) = create_access_log_channel(true, global::meter("kekw").i64_up_down_counter("lol").build());

        let manifest_path = path.join("manifest.json");
        let manifest = fs::read_to_string(manifest_path).unwrap();
        let manifest = serde_json::from_str(&manifest).unwrap();

        let extension = Extension {
            manifest,
            wasm_path: path.join("extension.wasm"),
        };

        let config = toml::from_str(&self.config.toml).unwrap();
        let hooks = WasmHooks::new(&SharedResources { access_log }, &config, Some(extension))
            .await
            .unwrap();

        self.runtime.hooks_extension = Some(hooks);

        self
    }

    pub fn with_mock_hooks(mut self, hooks: impl Into<DynamicHooks>) -> Self {
        self.runtime.hooks = Some(hooks.into());
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

    pub async fn try_build(self) -> Result<Gateway, String> {
        let Self {
            tmpdir,
            federated_sdl,
            mock_subgraphs,
            docker_subgraphs,
            virtual_subgraphs,
            config,
            runtime,
        } = self;

        let gateway_config = toml::from_str(&config.toml).expect("to be able to parse config");

        let subgraphs = Subgraphs::load(
            mock_subgraphs,
            docker_subgraphs,
            virtual_subgraphs
                .into_iter()
                .map(|(name, sdl)| Subgraph::Virtual { name, sdl })
                .collect(),
        )
        .await;

        let hooks_extension = runtime.hooks_extension.clone();
        let (engine, context) = self::engine::build(tmpdir.path(), federated_sdl, config, runtime, &subgraphs).await?;

        let router = self::router::build(engine.clone(), gateway_config, hooks_extension).await;

        Ok(Gateway {
            tmpdir: Arc::new(tmpdir),
            router,
            engine,
            context: Arc::new(context),
            subgraphs,
        })
    }
}
