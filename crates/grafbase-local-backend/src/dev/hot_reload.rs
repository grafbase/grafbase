use super::configurations::get_and_merge_configurations;
use super::subgraphs::{get_subgraph_sdls, SubgraphCache};
use super::FullGraphRef;
use crate::errors::BackendError;
use futures::lock::Mutex;
use gateway_config::Config;
use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use std::{path::PathBuf, time::Duration};
use tokio::runtime::Handle;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

const WATCHER_DEBOUNCE_TIME: Duration = Duration::from_secs(2);

fn watch_configuration_files(
    gateway_config_path: Option<&PathBuf>,
    graph_overrides_path: Option<&PathBuf>,
) -> Result<mpsc::Receiver<()>, notify::Error> {
    let (watcher_sender, watcher_receiver) = mpsc::channel::<()>(1);

    let runtime = Handle::current();

    let mut watcher = new_debouncer(WATCHER_DEBOUNCE_TIME, None, move |result: DebounceEventResult| {
        if result.is_err() {
            return;
        }

        let file_sender = watcher_sender.clone();
        runtime.block_on(async {
            let _ = file_sender.send(()).await;
        })
    })?;

    if let Some(gateway_config_path) = gateway_config_path {
        watcher.watch(gateway_config_path, RecursiveMode::NonRecursive)?;
    }

    if let Some(graph_overrides_path) = graph_overrides_path {
        watcher.watch(graph_overrides_path, RecursiveMode::NonRecursive)?;
    }

    // since the config watcher should live for the remainder of the program,
    // leak it instead of needing to make sure it isn't dropped
    Box::leak(Box::new(watcher));

    Ok(watcher_receiver)
}

pub async fn hot_reload(
    sender: mpsc::Sender<(String, Config)>,
    mut ready_receiver: Receiver<String>,
    graph_ref: Option<FullGraphRef>,
    subgraph_cache: SubgraphCache,
    gateway_config_path: Option<&'static PathBuf>,
    graph_overrides_path: Option<&'static PathBuf>,
) {
    // start hot reloading once the server is ready
    if ready_receiver.recv().await.is_err() {
        return;
    }

    if gateway_config_path.is_none() && graph_overrides_path.is_none() {
        // return early since we don't hot reload graphs from the API
        return;
    }

    let Ok(watcher_receiver) = watch_configuration_files(gateway_config_path, graph_overrides_path)
        .map_err(BackendError::SetUpWatcher)
        .inspect_err(|error| tracing::error!("{}", error.to_string().trim()))
    else {
        return;
    };

    let subgraph_cache = Mutex::new(subgraph_cache);

    let config_sender = sender.clone();

    tokio::spawn(async move {
        let mut stream = ReceiverStream::new(watcher_receiver);
        while stream.next().await.is_some() {
            let dev_configuration = match get_and_merge_configurations(gateway_config_path, graph_overrides_path).await
            {
                Ok(dev_configuration) => dev_configuration,
                Err(error) => {
                    tracing::error!("{}", error.to_string().trim());
                    continue;
                }
            };

            let subgraph_cache_guard = subgraph_cache.lock().await;
            let mut subgraphs = graphql_composition::Subgraphs::default();

            if graph_ref.is_some() {
                for (name, remote_subgraph) in subgraph_cache_guard
                    .remote
                    .iter()
                    .filter(|(name, _)| !dev_configuration.overridden_subgraphs.contains(**name))
                {
                    if let Err(error) = subgraphs
                        .ingest_str(&remote_subgraph.schema, name, &remote_subgraph.url)
                        .map_err(BackendError::IngestSubgraph)
                    {
                        tracing::error!("{}", error.to_string().trim());
                        continue;
                    };
                }
            }

            // we're not passing in the graph ref to avoid fetching the remote subgraphs again
            // as we have them cached
            match get_subgraph_sdls(None, &dev_configuration, &mut subgraphs, graph_overrides_path).await {
                Ok(value) => value,
                Err(error) => {
                    tracing::error!("{}", error.to_string().trim());
                    continue;
                }
            };

            if !subgraphs.is_empty() {
                let composition_result = graphql_composition::compose(&subgraphs);

                let federated_sdl = match composition_result.into_result() {
                    Ok(result) => {
                        match federated_graph::render_federated_sdl(&result).map_err(BackendError::ToFederatedSdl) {
                            Ok(sdl) => sdl,
                            Err(error) => {
                                tracing::error!("{}", error.to_string().trim());
                                continue;
                            }
                        }
                    }
                    Err(diagnostics) => {
                        tracing::error!(
                            "{}",
                            BackendError::Composition(diagnostics.iter_messages().collect::<Vec<_>>().join("\n"))
                                .to_string()
                                .trim()
                        );
                        continue;
                    }
                };

                tracing::info!("detected a configuation change, reloading");

                let _ = config_sender
                    .send((federated_sdl, dev_configuration.merged_configuration))
                    .await;
            }
        }

        Ok::<_, BackendError>(())
    });
}
