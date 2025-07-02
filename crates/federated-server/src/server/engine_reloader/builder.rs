use std::{path::PathBuf, pin::pin, sync::Arc};

use engine::CachedOperation;
use extension_catalog::ExtensionCatalog;
use futures_lite::{Stream, StreamExt};
use tokio::{sync::watch, task::JoinHandle};
use tokio_stream::wrappers::WatchStream;

use crate::{
    AccessToken,
    server::{
        GatewayEngineReloader,
        engine_reloader::Update,
        gateway::{self, GatewayRuntime, GraphDefinition},
        graph_fetch_method::GraphStream,
    },
};

#[derive(Default)]
pub struct GatewayEngineReloaderBuilder<'a> {
    config_receiver: Option<watch::Receiver<gateway_config::Config>>,
    graph_stream: Option<GraphStream>,
    hot_reload_config_path: Option<PathBuf>,
    access_token: Option<AccessToken>,
    extension_catalog: Option<&'a ExtensionCatalog>,
    logging_filter: Option<String>,
}

// New implementation for the builder
impl<'a> GatewayEngineReloaderBuilder<'a> {
    // Methods to set each parameter
    pub fn config_receiver(mut self, config_receiver: watch::Receiver<gateway_config::Config>) -> Self {
        self.config_receiver = Some(config_receiver);
        self
    }

    pub fn graph_stream(mut self, graph_stream: GraphStream) -> Self {
        self.graph_stream = Some(graph_stream);
        self
    }

    pub fn hot_reload_path(mut self, path: Option<PathBuf>) -> Self {
        self.hot_reload_config_path = path;
        self
    }

    pub fn access_token(mut self, token: Option<AccessToken>) -> Self {
        self.access_token = token;
        self
    }

    pub fn extension_catalog(mut self, catalog: &'a ExtensionCatalog) -> Self {
        self.extension_catalog = Some(catalog);
        self
    }

    pub fn logging_filter(mut self, filter: String) -> Self {
        self.logging_filter = Some(filter);
        self
    }

    // The build method consumes the builder and creates the GatewayEngineReloader
    pub async fn build(mut self) -> crate::Result<super::GatewayEngineReloader> {
        let gateway_config = self.config_receiver.take().ok_or_else(|| {
            crate::Error::InternalError("config_receiver not provided to GatewayEngineReloaderBuilder".to_string())
        })?;

        let mut graph_stream = self.graph_stream.take().ok_or_else(|| {
            crate::Error::InternalError("graph_stream not provided to GatewayEngineReloaderBuilder".to_string())
        })?;

        let extension_catalog = self.extension_catalog.take().ok_or_else(|| {
            crate::Error::InternalError("extension_catalog not provided to GatewayEngineReloaderBuilder".to_string())
        })?;

        let logging_filter = self.logging_filter.take().ok_or_else(|| {
            crate::Error::InternalError("logging_filter not provided to GatewayEngineReloaderBuilder".to_string())
        })?;

        tracing::debug!("Waiting for a graph...");
        let Some(graph_definition) = graph_stream.next().await else {
            return Err(crate::Error::InternalError(
                "No initial graph definition provided".into(),
            ));
        };

        tracing::debug!("Creating the engine");

        let engine = EngineBuilder::default()
            .config(gateway_config.borrow().clone())
            .graph_definition(graph_definition.clone())
            .hot_reload_config_path(self.hot_reload_config_path.clone())
            .access_token(self.access_token.clone())
            .operations_to_warm(vec![])
            .extension_catalog(extension_catalog)
            .logging_filter(logging_filter.clone())
            .build()
            .await?;

        let (engine_sender, engine_watcher) = watch::channel(engine);

        let hot_reload_config_path = self.hot_reload_config_path;
        let access_token = self.access_token;

        tokio::spawn(async move {
            let graph_stream = graph_stream.map(Update::Graph);

            let config_stream =
                WatchStream::from_changes(gateway_config.clone()).map(|config| Update::Config(Box::new(config)));

            let updates = graph_stream.race(config_stream);
            let current_config = gateway_config.borrow().clone();

            UpdateLoopBuilder::new(updates)
                .current_config(current_config)
                .graph_definition(graph_definition)
                .hot_reload_config_path(hot_reload_config_path)
                .access_token(access_token)
                .engine_sender(engine_sender)
                .logging_filter(logging_filter)
                .build()
                .await
        });

        Ok(GatewayEngineReloader { engine_watcher })
    }
}

#[derive(Clone, Default)]
pub struct EngineBuilder<'a> {
    config: Option<gateway_config::Config>,
    graph_definition: Option<GraphDefinition>,
    hot_reload_config_path: Option<PathBuf>,
    access_token: Option<AccessToken>,
    operations_to_warm: Vec<Arc<CachedOperation>>,
    extension_catalog: Option<&'a ExtensionCatalog>,
    logging_filter: Option<String>,
}

impl<'a> EngineBuilder<'a> {
    pub fn config(mut self, config: gateway_config::Config) -> Self {
        self.config = Some(config);
        self
    }

    pub fn graph_definition(mut self, graph_definition: GraphDefinition) -> Self {
        self.graph_definition = Some(graph_definition);
        self
    }

    pub fn hot_reload_config_path(mut self, path: Option<PathBuf>) -> Self {
        self.hot_reload_config_path = path;
        self
    }

    pub fn access_token(mut self, token: Option<AccessToken>) -> Self {
        self.access_token = token;
        self
    }

    pub fn operations_to_warm(mut self, operations: Vec<Arc<CachedOperation>>) -> Self {
        self.operations_to_warm = operations;
        self
    }

    pub fn extension_catalog(mut self, catalog: &'a ExtensionCatalog) -> Self {
        self.extension_catalog = Some(catalog);
        self
    }

    pub fn logging_filter(mut self, filter: String) -> Self {
        self.logging_filter = Some(filter);
        self
    }

    pub async fn build(self) -> crate::Result<Arc<engine::Engine<GatewayRuntime>>> {
        let config = self
            .config
            .ok_or_else(|| crate::Error::InternalError("config not provided to EngineBuilder".to_string()))?;

        let graph_definition = self
            .graph_definition
            .ok_or_else(|| crate::Error::InternalError("graph_definition not provided to EngineBuilder".to_string()))?;

        let logging_filter = self
            .logging_filter
            .ok_or_else(|| crate::Error::InternalError("logging_filter not provided to EngineBuilder".to_string()))?;

        let extension_catalog = self.extension_catalog;

        let engine = gateway::generate(
            graph_definition,
            &config,
            self.hot_reload_config_path,
            self.access_token.as_ref(),
            extension_catalog,
            logging_filter,
        )
        .await?;

        let engine = Arc::new(engine);

        engine.warm(self.operations_to_warm).await;

        Ok(engine)
    }
}

pub struct UpdateLoopBuilder<S> {
    updates: S,
    current_config: Option<gateway_config::Config>,
    graph_definition: Option<GraphDefinition>,
    hot_reload_config_path: Option<PathBuf>,
    access_token: Option<AccessToken>,
    engine_sender: Option<watch::Sender<Arc<engine::Engine<GatewayRuntime>>>>,
    logging_filter: Option<String>,
}

impl<S> UpdateLoopBuilder<S>
where
    S: Stream<Item = Update> + Unpin,
{
    pub fn new(updates: S) -> Self {
        Self {
            updates,
            current_config: None,
            graph_definition: None,
            hot_reload_config_path: None,
            access_token: None,
            engine_sender: None,
            logging_filter: None,
        }
    }

    pub fn current_config(mut self, config: gateway_config::Config) -> Self {
        self.current_config = Some(config);
        self
    }

    pub fn graph_definition(mut self, graph_definition: GraphDefinition) -> Self {
        self.graph_definition = Some(graph_definition);
        self
    }

    pub fn hot_reload_config_path(mut self, path: Option<PathBuf>) -> Self {
        self.hot_reload_config_path = path;
        self
    }

    pub fn access_token(mut self, token: Option<AccessToken>) -> Self {
        self.access_token = token;
        self
    }

    pub fn engine_sender(mut self, sender: watch::Sender<Arc<engine::Engine<GatewayRuntime>>>) -> Self {
        self.engine_sender = Some(sender);
        self
    }

    pub fn logging_filter(mut self, filter: String) -> Self {
        self.logging_filter = Some(filter);
        self
    }

    pub async fn build(self) {
        let mut current_config = self
            .current_config
            .expect("current_config not provided to UpdateLoopBuilder");

        let mut graph_definition = self
            .graph_definition
            .expect("graph_definition not provided to UpdateLoopBuilder");

        let engine_sender = self
            .engine_sender
            .expect("engine_sender not provided to UpdateLoopBuilder");

        let logging_filter = self
            .logging_filter
            .expect("logging_filter not provided to UpdateLoopBuilder");

        let mut in_progress_reload: Option<JoinHandle<()>> = None;

        let mut updates = pin!(self.updates);

        while let Some(update) = updates.next().await {
            if let Some(in_progress_reload) = in_progress_reload.take() {
                in_progress_reload.abort();
            }

            match update {
                Update::Graph(new_graph) => graph_definition = new_graph,
                Update::Config(config) => current_config = *config,
            }

            in_progress_reload = Some(tokio::spawn({
                let hot_reload_config_path = self.hot_reload_config_path.clone();
                let access_token = self.access_token.clone();
                let current_config = current_config.clone();
                let graph_definition = graph_definition.clone();
                let engine_sender = engine_sender.clone();
                let logging_filter = logging_filter.clone();

                async move {
                    let operations_to_warm = super::extract_operations_to_warm(&current_config, &engine_sender);

                    let result = EngineBuilder::default()
                        .config(current_config)
                        .graph_definition(graph_definition)
                        .hot_reload_config_path(hot_reload_config_path)
                        .access_token(access_token)
                        .operations_to_warm(operations_to_warm)
                        .logging_filter(logging_filter)
                        .build()
                        .await;

                    match result {
                        Ok(engine) => {
                            if let Err(err) = engine_sender.send(engine) {
                                tracing::error!("Could not send engine: {err:?}");
                            }
                        }
                        Err(err) => {
                            tracing::error!("Could not build engine from latest graph: {err}")
                        }
                    }
                }
            }));
        }
    }
}
