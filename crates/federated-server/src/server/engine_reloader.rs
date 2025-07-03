use std::{path::PathBuf, sync::Arc};

use engine::CachedOperation;
use extension_catalog::ExtensionCatalog;
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};

use super::{
    AccessToken,
    events::UpdateEvent,
    gateway::{self, EngineBuildContext, EngineSender, EngineWatcher, GatewayRuntime, GraphDefinition},
};

/// Configuration for the GatewayEngineReloader.
pub struct EngineReloaderConfig<'a> {
    /// The channel receiver for update events
    pub update_receiver: mpsc::Receiver<UpdateEvent>,

    /// The initial gateway configuration
    pub initial_config: gateway_config::Config,

    /// The extension catalog for the engine
    pub extension_catalog: &'a ExtensionCatalog,

    /// The logging filter string
    pub logging_filter: String,

    /// Optional path for hot reload configuration
    pub hot_reload_config_path: Option<PathBuf>,

    /// Optional access token for authenticated operations
    pub access_token: Option<AccessToken>,
}

/// Handles graph and config updates by constructing a new engine
pub(super) struct GatewayEngineReloader {
    engine_watcher: EngineWatcher<GatewayRuntime>,
}

impl GatewayEngineReloader {
    /// Spawns a new engine reloader with the given configuration.
    ///
    /// This method:
    /// 1. Waits for the initial graph definition
    /// 2. Builds the initial engine
    /// 3. Spawns a background task that listens for updates and rebuilds the engine
    pub async fn spawn(
        EngineReloaderConfig {
            mut update_receiver,
            initial_config,
            extension_catalog,
            logging_filter,
            hot_reload_config_path,
            access_token,
        }: EngineReloaderConfig<'_>,
    ) -> crate::Result<Self> {
        let mut current_config = initial_config;

        tracing::debug!("Waiting for the initial graph...");

        let mut graph_definition = loop {
            match update_receiver.recv().await {
                Some(UpdateEvent::Graph(graph_def)) => break graph_def,
                Some(UpdateEvent::Config(new_config)) => {
                    // Update config if we receive it before the initial graph
                    current_config = *new_config;
                    continue;
                }
                None => {
                    return Err(crate::Error::InternalError(
                        "Update channel closed before initial graph definition".into(),
                    ));
                }
            }
        };

        // Build the initial engine
        tracing::debug!("Creating the initial engine");

        let initial_context = EngineBuildContext {
            gateway_config: &current_config,
            hot_reload_config_path: hot_reload_config_path.as_ref(),
            access_token: access_token.as_ref(),
            extension_catalog: Some(extension_catalog),
            logging_filter: &logging_filter,
        };

        let engine = build_engine(initial_context, graph_definition.clone(), vec![]).await?;
        let (engine_sender, engine_watcher) = watch::channel(engine);

        tokio::spawn(async move {
            let mut in_progress_reload: Option<JoinHandle<()>> = None;

            while let Some(update) = update_receiver.recv().await {
                // Abort any in-progress reload
                if let Some(reload) = in_progress_reload.take() {
                    reload.abort();
                }

                match update {
                    UpdateEvent::Graph(new_graph) => graph_definition = new_graph,
                    UpdateEvent::Config(new_config) => current_config = *new_config,
                }

                in_progress_reload = Some(tokio::spawn({
                    let hot_reload_config_path = hot_reload_config_path.clone();
                    let access_token = access_token.clone();
                    let current_config = current_config.clone();
                    let graph_definition = graph_definition.clone();
                    let engine_sender = engine_sender.clone();
                    let logging_filter = logging_filter.clone();

                    async move {
                        let operations_to_warm = extract_operations_to_warm(&current_config, &engine_sender);

                        let context = EngineBuildContext {
                            gateway_config: &current_config,
                            hot_reload_config_path: hot_reload_config_path.as_ref(),
                            access_token: access_token.as_ref(),
                            extension_catalog: None, // Will be created by gateway::generate if needed
                            logging_filter: &logging_filter,
                        };

                        match build_engine(context, graph_definition, operations_to_warm).await {
                            Ok(new_engine) => {
                                if let Err(err) = engine_sender.send(new_engine) {
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

            tracing::info!("Update loop terminated");
        });

        Ok(GatewayEngineReloader { engine_watcher })
    }

    pub fn engine_watcher(&self) -> EngineWatcher<GatewayRuntime> {
        self.engine_watcher.clone()
    }
}

/// Helper function that builds a new engine instance.
async fn build_engine(
    context: EngineBuildContext<'_>,
    graph_definition: GraphDefinition,
    operations_to_warm: Vec<Arc<CachedOperation>>,
) -> crate::Result<Arc<engine::Engine<GatewayRuntime>>> {
    let engine = gateway::generate(context, graph_definition).await?;
    let engine = Arc::new(engine);

    engine.warm(operations_to_warm).await;

    Ok(engine)
}

fn extract_operations_to_warm(
    config: &gateway_config::Config,
    engine_sender: &EngineSender,
) -> Vec<Arc<CachedOperation>> {
    if !config.operation_caching.enabled || !config.operation_caching.warm_on_reload {
        return Vec::new();
    }

    let (operations, cache_count) = {
        let cache = &engine_sender.borrow().runtime.operation_cache;
        (cache.values().collect(), cache.entry_count())
    };

    if config.operation_caching.warming_percent >= 100 {
        return operations;
    }

    operations
        .into_iter()
        .take(cache_count * (config.operation_caching.warming_percent as usize / 100))
        .collect()
}
