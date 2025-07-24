mod bench;
mod engine;
mod router;

use std::{any::TypeId, collections::HashSet, fmt::Display, sync::Arc};

use crate::{TestTrustedDocument, mock_trusted_documents::MockTrustedDocumentsClient};
pub use bench::*;
use futures::{FutureExt, future::BoxFuture};
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

    pub fn with_mock_fetcher(mut self, fetcher: impl Into<DynamicFetcher>) -> Self {
        self.runtime.fetcher = Some(fetcher.into());
        self
    }
    //-- Runtime customization --

    pub async fn build(self) -> Gateway {
        self.build_inner().await.unwrap()
    }

    pub async fn try_build(self) -> Result<Gateway, String> {
        self.build_inner().await.map_err(|err| err.to_string())
    }

    pub async fn build_inner(self) -> anyhow::Result<Gateway> {
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

        let (engine, extension_catalog) =
            self::engine::build(tmpdir.path(), federated_sdl, config, runtime, &subgraphs).await?;
        let router = self::router::build(engine.clone(), gateway_config, extension_catalog).await;

        Ok(Gateway {
            tmpdir: Arc::new(tmpdir),
            router,
            engine,
            subgraphs,
        })
    }
}
