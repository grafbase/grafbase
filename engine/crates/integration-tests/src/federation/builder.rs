mod bench;
mod engine;
mod router;
mod test_runtime;

use std::{any::TypeId, collections::HashSet};

use crate::{mock_trusted_documents::MockTrustedDocumentsClient, TestTrustedDocument};
pub use bench::*;
use futures::{future::BoxFuture, FutureExt};
use graphql_mocks::MockGraphQlServer;
use runtime::{fetch::dynamic::DynamicFetcher, hooks::DynamicHooks, trusted_documents_client};
pub use test_runtime::*;

use super::{subgraph::Subgraphs, DockerSubgraph, TestGateway};

enum ConfigSource {
    Sdl(String),
    Toml(String),
    SdlWebsocket,
}

#[must_use]
#[derive(Default)]
pub struct TestGatewayBuilder {
    federated_sdl: Option<String>,
    mock_subgraphs: Vec<(TypeId, String, BoxFuture<'static, MockGraphQlServer>)>,
    docker_subgraphs: HashSet<DockerSubgraph>,
    config_source: Option<ConfigSource>,
    runtime: TestRuntime,
}

pub trait EngineV2Ext {
    fn builder() -> TestGatewayBuilder {
        TestGatewayBuilder::default()
    }
}

impl EngineV2Ext for engine_v2::Engine<TestRuntime> {}

impl TestGatewayBuilder {
    pub fn with_sdl_config(mut self, sdl: impl Into<String>) -> Self {
        assert!(self.config_source.is_none(), "overwriting config!");
        self.config_source = Some(ConfigSource::Sdl(sdl.into()));
        self
    }

    pub fn with_toml_config(mut self, toml: impl Into<String>) -> Self {
        assert!(self.config_source.is_none(), "overwriting config!");
        self.config_source = Some(ConfigSource::Toml(toml.into()));
        self
    }

    pub fn with_sdl_websocket_config(mut self) -> Self {
        assert!(self.config_source.is_none(), "overwriting config!");
        self.config_source = Some(ConfigSource::SdlWebsocket);
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

    /// Will bypass the composition of subgraphs and be used at its stead.
    pub fn with_federated_sdl(mut self, sdl: &str) -> Self {
        self.federated_sdl = Some(sdl.to_string());
        self
    }

    //-- Runtime customization --
    // Prefer passing through either the TOML / SDL config when relevant, see update_runtime_with_toml_config
    //--

    pub fn with_mock_trusted_documents(mut self, branch_id: String, documents: Vec<TestTrustedDocument>) -> Self {
        self.runtime.trusted_documents = trusted_documents_client::Client::new(MockTrustedDocumentsClient {
            _branch_id: branch_id,
            documents,
        });
        self
    }

    pub fn with_mock_hooks(mut self, hooks: impl Into<DynamicHooks>) -> Self {
        self.runtime.hooks = hooks.into();
        self
    }

    pub fn with_mock_fetcher(mut self, fetcher: impl Into<DynamicFetcher>) -> Self {
        self.runtime.fetcher = fetcher.into();
        self
    }
    //-- Runtime customization --

    pub async fn build(self) -> TestGateway {
        let Self {
            federated_sdl,
            mock_subgraphs,
            docker_subgraphs,
            config_source,
            runtime,
        } = self;
        let subgraphs = Subgraphs::load(mock_subgraphs, docker_subgraphs).await;

        let (engine, context) = self::engine::build(federated_sdl, config_source, runtime, &subgraphs).await;
        let router = self::router::build(engine.clone());

        TestGateway {
            router,
            engine,
            context,
            subgraphs,
        }
    }
}
