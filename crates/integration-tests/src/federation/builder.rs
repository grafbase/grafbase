mod bench;
mod engine;
mod router;

use std::{any::TypeId, collections::HashSet, fmt::Display, sync::Arc};

use crate::{mock_trusted_documents::MockTrustedDocumentsClient, TestTrustedDocument};
pub use bench::*;
use futures::{future::BoxFuture, FutureExt};
use graphql_mocks::MockGraphQlServer;
use runtime::{
    fetch::dynamic::DynamicFetcher,
    hooks::DynamicHooks,
    trusted_documents_client::{self, TrustedDocumentsEnforcementMode},
};

use super::{subgraph::Subgraphs, DockerSubgraph, TestExtensions, TestGateway, TestRuntime};

#[derive(Default)]
struct TestConfig {
    toml: String,
    add_websocket_url: bool,
}

#[must_use]
#[derive(Default)]
pub struct TestGatewayBuilder {
    federated_sdl: Option<String>,
    mock_subgraphs: Vec<(TypeId, String, BoxFuture<'static, MockGraphQlServer>)>,
    docker_subgraphs: HashSet<DockerSubgraph>,
    config: TestConfig,

    trusted_documents: Option<trusted_documents_client::Client>,
    hooks: Option<DynamicHooks>,
    fetcher: Option<DynamicFetcher>,
    extensions: TestExtensions,
}

pub trait EngineExt {
    fn builder() -> TestGatewayBuilder {
        TestGatewayBuilder::default()
    }
}

impl EngineExt for ::engine::Engine<TestRuntime> {}

impl TestGatewayBuilder {
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
        self.trusted_documents = Some(trusted_documents_client::Client::new(MockTrustedDocumentsClient {
            documents,
            enforcement_mode,
        }));
        self
    }

    pub fn with_extensions(mut self, f: impl FnOnce(TestExtensions) -> TestExtensions) -> Self {
        self.extensions = f(self.extensions);
        self
    }

    pub fn with_mock_hooks(mut self, hooks: impl Into<DynamicHooks>) -> Self {
        self.hooks = Some(hooks.into());
        self
    }

    pub fn with_mock_fetcher(mut self, fetcher: impl Into<DynamicFetcher>) -> Self {
        self.fetcher = Some(fetcher.into());
        self
    }
    //-- Runtime customization --

    pub async fn build(self) -> TestGateway {
        let Self {
            federated_sdl,
            mock_subgraphs,
            docker_subgraphs,
            config,
            trusted_documents,
            hooks,
            fetcher,
            extensions,
        } = self;

        let gateway_config = toml::from_str(&config.toml).expect("to be able to parse config");
        let mut runtime = TestRuntime::new(&gateway_config);
        runtime.extensions = extensions;

        if let Some(trusted_documents) = trusted_documents {
            runtime.trusted_documents = trusted_documents;
        }

        if let Some(hooks) = hooks {
            runtime.hooks = hooks;
        }

        if let Some(fetcher) = fetcher {
            runtime.fetcher = fetcher;
        }

        let subgraphs = Subgraphs::load(mock_subgraphs, docker_subgraphs).await;

        let (engine, context) = self::engine::build(federated_sdl, config, runtime, &subgraphs).await;
        let router = self::router::build(engine.clone(), &gateway_config);

        TestGateway {
            router,
            engine,
            context: Arc::new(context),
            subgraphs,
        }
    }
}
