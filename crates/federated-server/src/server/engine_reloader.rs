use std::{path::PathBuf, sync::Arc};

use engine::{CachedOperation, Engine};
use futures_lite::{pin, StreamExt};
use runtime_local::wasi::{extensions::WasiExtensions, hooks::HooksWasi};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
use tokio_stream::{wrappers::WatchStream, Stream};

use crate::server::gateway;

use super::{
    gateway::{EngineSender, EngineWatcher, GatewayRuntime, GraphDefinition},
    graph_fetch_method::GraphStream,
};

/// Handles graph and config updates by constructing a new engine
pub(super) struct EngineReloader {
    engine_watcher: EngineWatcher,
}

pub(crate) type GraphSender = mpsc::Sender<GraphDefinition>;

enum Update {
    Graph(GraphDefinition),
    Config(gateway_config::Config),
}

impl EngineReloader {
    pub async fn spawn(
        // A receiver that passes the config in.  In the gateway this is usually
        // static, but in federated dev it will change on reloads.
        gateway_config: watch::Receiver<gateway_config::Config>,
        mut graph_stream: GraphStream,
        // The config path that should be used to hot reloading in the gateway.
        // In federated dev this is None.  We should probably merge this
        // functionality into gateway_config above at some point...
        hot_reload_config_path: Option<PathBuf>,
        hooks: HooksWasi,
        extensions: WasiExtensions,
    ) -> crate::Result<Self> {
        let context = Context {
            hot_reload_config_path,
            hooks,
            extensions,
        };

        tracing::debug!("Waiting for a graph...");
        let Some(graph_definition) = graph_stream.next().await else {
            // This shouldn't really happen, but someone could mess up
            return Err(crate::Error::InternalError(
                "No initial graph definition provided".into(),
            ));
        };

        tracing::debug!("Creating the engine");
        let engine = build_new_engine(
            gateway_config.borrow().clone(),
            graph_definition.clone(),
            context.clone(),
            vec![],
        )
        .await?;

        let (engine_sender, engine_watcher) = watch::channel(engine);

        tokio::spawn(async move {
            let graph_stream = graph_stream.map(Update::Graph);
            let config_stream = WatchStream::from_changes(gateway_config.clone()).map(Update::Config);
            let updates = graph_stream.race(config_stream);
            let current_config = gateway_config.borrow().clone();

            update_loop(updates, current_config, graph_definition, context, engine_sender).await
        });

        Ok(EngineReloader { engine_watcher })
    }

    pub fn engine_watcher(&self) -> EngineWatcher {
        self.engine_watcher.clone()
    }
}

#[derive(Clone)]
struct Context {
    hot_reload_config_path: Option<PathBuf>,
    hooks: HooksWasi,
    extensions: WasiExtensions,
}

async fn update_loop(
    updates: impl Stream<Item = Update>,
    mut current_config: gateway_config::Config,
    mut graph_definition: GraphDefinition,
    context: Context,
    engine_sender: EngineSender,
) {
    let mut in_progress_reload: Option<JoinHandle<()>> = None;

    pin!(updates);

    while let Some(update) = updates.next().await {
        if let Some(in_progress_reload) = in_progress_reload.take() {
            in_progress_reload.abort();
        }

        match update {
            Update::Graph(new_graph) => graph_definition = new_graph,
            Update::Config(config) => current_config = config,
        }

        in_progress_reload = Some(tokio::spawn({
            let context = context.clone();
            let current_config = current_config.clone();
            let graph_definition = graph_definition.clone();
            let engine_sender = engine_sender.clone();

            async move {
                let operations_to_warm = extract_operations_to_warm(&current_config, &engine_sender);

                match build_new_engine(current_config, graph_definition, context, operations_to_warm).await {
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

async fn build_new_engine(
    config: gateway_config::Config,
    graph_definition: GraphDefinition,
    context: Context,
    operations_to_warm: Vec<Arc<CachedOperation>>,
) -> crate::Result<Arc<Engine<GatewayRuntime>>> {
    let engine = gateway::generate(
        graph_definition,
        &config,
        context.hot_reload_config_path,
        context.hooks,
        context.extensions,
    )
    .await?;

    let engine = Arc::new(engine);

    engine.warm(operations_to_warm).await;

    Ok(engine)
}

fn extract_operations_to_warm(
    config: &gateway_config::Config,
    engine_sender: &EngineSender,
) -> Vec<Arc<CachedOperation>> {
    if !config.operation_caching.enabled || !config.operation_caching.warm_on_reload {
        return vec![];
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
