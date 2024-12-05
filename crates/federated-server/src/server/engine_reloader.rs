use std::{path::PathBuf, sync::Arc};

use engine::Engine;
use futures_lite::{pin, StreamExt};
use runtime_local::HooksWasi;
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
use tokio_stream::{
    wrappers::{ReceiverStream, WatchStream},
    Stream,
};

use crate::server::gateway;

use super::gateway::{EngineSender, EngineWatcher, GatewayRuntime, GraphDefinition};

/// Handles graph and config updates by constructing a new engine
pub(super) struct EngineReloader {
    graph_sender: GraphSender,
    engine_watcher: EngineWatcher,
}

pub(crate) type GraphSender = mpsc::Sender<GraphDefinition>;

enum Update {
    Graph(GraphDefinition),
    Config(gateway_config::Config),
}

impl EngineReloader {
    pub async fn spawn(
        gateway_config: watch::Receiver<gateway_config::Config>,
        hot_reload_config_path: Option<PathBuf>,
        hooks: HooksWasi,
    ) -> crate::Result<Self> {
        let (graph_sender, mut graph_receiver) = mpsc::channel::<GraphDefinition>(4);
        let (engine_sender, engine_watcher) = watch::channel::<Option<Arc<Engine<GatewayRuntime>>>>(None);

        let context = Context {
            hot_reload_config_path,
            hooks,
            engine_sender,
        };

        let Some(graph_definition) = graph_receiver.recv().await else {
            // This shouldn't really happen, but someone could mess up
            return Err(crate::Error::InternalError("No initial graph setup".into()));
        };

        build_new_engine(
            gateway_config.borrow().clone(),
            graph_definition.clone(),
            context.clone(),
        )
        .await?;

        tokio::spawn(async move {
            let graph_stream = ReceiverStream::new(graph_receiver).map(Update::Graph);
            let config_stream = WatchStream::from_changes(gateway_config.clone()).map(Update::Config);
            let updates = graph_stream.race(config_stream);
            let current_config = gateway_config.borrow().clone();

            update_loop(updates, current_config, graph_definition, context).await
        });

        Ok(EngineReloader {
            graph_sender,
            engine_watcher,
        })
    }

    pub fn graph_sender(&self) -> GraphSender {
        self.graph_sender.clone()
    }

    pub fn engine_watcher(&self) -> EngineWatcher {
        self.engine_watcher.clone()
    }
}

#[derive(Clone)]
struct Context {
    hot_reload_config_path: Option<PathBuf>,
    hooks: HooksWasi,
    engine_sender: EngineSender,
}

async fn update_loop(
    updates: impl Stream<Item = Update>,
    mut current_config: gateway_config::Config,
    mut graph_definition: GraphDefinition,
    context: Context,
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

            async move {
                if let Err(err) = build_new_engine(current_config, graph_definition, context).await {
                    tracing::error!("Could not build engine from latest graph: {err}")
                }
            }
        }));
    }
}

async fn build_new_engine(
    current_config: gateway_config::Config,
    graph_definition: GraphDefinition,
    context: Context,
) -> crate::Result<()> {
    let engine = gateway::generate(
        graph_definition,
        &current_config,
        context.hot_reload_config_path,
        context.hooks,
    )
    .await?;

    let engine = Arc::new(engine);

    context.engine_sender.send(Some(engine))?;

    Ok(())
}
