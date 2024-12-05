use super::configurations::{get_and_merge_configurations, DevConfiguration};
use super::subgraphs::{get_subgraph_sdls, SubgraphCache};
use crate::dev::subgraphs::CachedIntrospectedSubgraph;
use crate::errors::BackendError;
use gateway_config::Config;
use grafbase_graphql_introspection::introspect;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, RecommendedCache};
use std::collections::HashSet;
use std::sync::Arc;
use std::{path::PathBuf, time::Duration};
use tokio::runtime::Handle;
use tokio::sync::broadcast::Receiver;
use tokio::sync::{mpsc, watch};
use tokio::time::MissedTickBehavior;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tokio_util::sync::CancellationToken;

struct SubgraphWatcher {
    watcher: Option<Debouncer<RecommendedWatcher, RecommendedCache>>,
    cancellation_token: Option<CancellationToken>,
}

impl SubgraphWatcher {
    fn new() -> Self {
        Self {
            watcher: None,
            cancellation_token: None,
        }
    }

    fn stop(&mut self) {
        self.watcher = None;
        if let Some(ref poller_cancellation_token) = self.cancellation_token {
            poller_cancellation_token.cancel();
        }
        self.cancellation_token = None;
    }

    #[allow(clippy::too_many_arguments)]
    fn start(
        &mut self,
        sender: mpsc::Sender<String>,
        subgraph_cache: Arc<SubgraphCache>,
        overridden_subgraphs: Arc<HashSet<String>>,
        merged_configuration: Arc<Config>,
        graph_overrides_path: Option<&'static PathBuf>,
    ) -> Result<(), BackendError> {
        // skip if there's no local subgraphs
        if overridden_subgraphs.is_empty() {
            return Ok(());
        }

        self.cancellation_token = Some(CancellationToken::new());

        self.spawn_introspection_poller(
            sender.clone(),
            subgraph_cache.clone(),
            overridden_subgraphs.clone(),
            merged_configuration.clone(),
            graph_overrides_path,
        )?;

        self.spawn_schema_file_watcher(
            sender,
            subgraph_cache,
            overridden_subgraphs,
            merged_configuration,
            graph_overrides_path,
        )
    }

    fn spawn_introspection_poller(
        &mut self,
        sender: mpsc::Sender<String>,
        subgraph_cache: Arc<SubgraphCache>,
        overridden_subgraphs: Arc<HashSet<String>>,
        merged_configuration: Arc<Config>,
        graph_overrides_path: Option<&'static PathBuf>,
    ) -> Result<(), BackendError> {
        let introspection_urls = merged_configuration
            .subgraphs
            .iter()
            .filter_map(|(name, subgraph)| {
                subgraph.introspection_url.as_ref().map(|introspection_url| {
                    (
                        name,
                        introspection_url.as_ref(),
                        subgraph.introspection_headers.as_ref(),
                    )
                })
            })
            .collect::<Vec<_>>();

        if introspection_urls.is_empty() {
            return Ok(());
        }

        let poller_cancellation_token = self.cancellation_token.as_ref().expect("must exist").child_token();

        let reload_cancellation_token = poller_cancellation_token.child_token();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(WATCHER_DEBOUNCE_DURATION);

            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            'poller: loop {
                tokio::select! {
                    _ = interval.tick() => {}
                    _ = poller_cancellation_token.cancelled() => { break 'poller; }
                }

                let mut cached_local_subgraphs = subgraph_cache.local.lock().await;

                let futures = cached_local_subgraphs
                    .iter()
                    .map(|(name, cached_local_subgraph)| async move {
                        // TODO: this also parses and prettifies, expose internal functionality
                        let sdl = introspect(
                            cached_local_subgraph.introspection_url.as_str(),
                            &cached_local_subgraph.introspection_headers,
                        )
                        .await
                        .map_err(|_| {
                            BackendError::IntrospectSubgraph(cached_local_subgraph.introspection_url.to_string())
                        })?;
                        let changed = sdl != cached_local_subgraph.sdl;
                        if changed {
                            Ok::<_, BackendError>(Some((
                                name.clone(),
                                CachedIntrospectedSubgraph {
                                    sdl,
                                    ..(*cached_local_subgraph).clone()
                                },
                            )))
                        } else {
                            Ok::<_, BackendError>(None)
                        }
                    });

                let results = match futures::future::try_join_all(futures).await {
                    Ok(results) => results,
                    Err(error) => {
                        tracing::error!("{}", error.to_string().trim());
                        continue;
                    }
                };

                let mut changed = false;

                for (name, changed_subgraph) in results.into_iter().flatten() {
                    changed = true;
                    cached_local_subgraphs.insert(name, changed_subgraph);
                }

                if changed {
                    // TODO: use the subgraph cache rather than introspecting
                    // (we'll need to prevent schema file and url reloads running at the same time to prevent stale data)
                    match reload_subgraphs(
                        sender.clone(),
                        subgraph_cache.clone(),
                        overridden_subgraphs.clone(),
                        merged_configuration.clone(),
                        graph_overrides_path,
                        Some(reload_cancellation_token.child_token()),
                    )
                    .await
                    {
                        Ok(_) => tracing::info!("detected a subgraph change, reloading"),
                        Err(error) => tracing::error!("{}", error.to_string().trim()),
                    }
                }
            }
        });

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_schema_file_watcher(
        &mut self,
        sender: mpsc::Sender<String>,
        subgraph_cache: Arc<SubgraphCache>,
        overridden_subgraphs: Arc<HashSet<String>>,
        merged_configuration: Arc<Config>,
        graph_overrides_path: Option<&'static PathBuf>,
    ) -> Result<(), BackendError> {
        let schema_file_paths = merged_configuration
            .subgraphs
            .iter()
            .filter_map(|(_name, subgraph)| subgraph.schema_path.as_ref())
            .collect::<Vec<_>>();

        if schema_file_paths.is_empty() {
            return Ok(());
        }

        let runtime_handle = Handle::current();

        let watcher_cancellation_token = self.cancellation_token.as_ref().expect("must exist").child_token();

        let watcher_merged_configuration = merged_configuration.clone();
        let mut watcher = new_debouncer(WATCHER_DEBOUNCE_DURATION, None, move |result: DebounceEventResult| {
            if result.is_err() {
                return;
            }
            let subgraph_cache = subgraph_cache.clone();
            let overridden_subgraphs = overridden_subgraphs.clone();
            let merged_configuration = watcher_merged_configuration.clone();
            let sender = sender.clone();

            if watcher_cancellation_token.is_cancelled() {
                return;
            }

            let reload_cancellation_token = watcher_cancellation_token.child_token();

            runtime_handle.block_on(async move {
                match reload_subgraphs(
                    sender.clone(),
                    subgraph_cache,
                    overridden_subgraphs,
                    merged_configuration,
                    graph_overrides_path,
                    Some(reload_cancellation_token),
                )
                .await
                {
                    Ok(_) => tracing::info!("detected a subgraph change, reloading"),
                    Err(error) => tracing::error!("{}", error.to_string().trim()),
                }
            });
        })
        .map_err(BackendError::SetUpWatcher)?;

        for path in schema_file_paths {
            watcher
                .watch(path, RecursiveMode::NonRecursive)
                .map_err(BackendError::SetUpWatcher)?;
        }

        self.watcher = Some(watcher);

        Ok(())
    }
}

const WATCHER_DEBOUNCE_DURATION: Duration = Duration::from_secs(2);

fn watch_configuration_files(
    gateway_config_path: Option<&PathBuf>,
    graph_overrides_path: Option<&PathBuf>,
) -> Result<mpsc::Receiver<()>, notify::Error> {
    let (watcher_sender, watcher_receiver) = mpsc::channel::<()>(1);

    let runtime = Handle::current();

    let mut watcher = new_debouncer(WATCHER_DEBOUNCE_DURATION, None, move |result: DebounceEventResult| {
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

    // since the config watcher should live for the remainder of the cli run,
    // leak it instead of needing to make sure it isn't dropped
    Box::leak(Box::new(watcher));

    Ok(watcher_receiver)
}

pub async fn hot_reload(
    config_sender: watch::Sender<Config>,
    sdl_sender: mpsc::Sender<String>,
    mut ready_receiver: Receiver<String>,
    subgraph_cache: Arc<SubgraphCache>,
    gateway_config_path: Option<&'static PathBuf>,
    graph_overrides_path: Option<&'static PathBuf>,
    dev_configuration: DevConfiguration,
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

    let mut subgraph_watcher = SubgraphWatcher::new();

    let overridden_subgraphs = Arc::new(dev_configuration.overridden_subgraphs);
    let merged_configuration = Arc::new(dev_configuration.merged_configuration);

    // don't skip the config reloader if
    // the subgraph watcher encountered an error
    let _ = subgraph_watcher
        .start(
            sdl_sender.clone(),
            subgraph_cache.clone(),
            overridden_subgraphs,
            merged_configuration,
            graph_overrides_path,
        )
        .inspect_err(|error| tracing::error!("{}", error.to_string().trim()));

    tokio::spawn(async move {
        let mut stream = ReceiverStream::new(watcher_receiver);
        while stream.next().await.is_some() {
            let subgraph_cache = subgraph_cache.clone();
            subgraph_watcher.stop();

            let dev_configuration = match get_and_merge_configurations(gateway_config_path, graph_overrides_path).await
            {
                Ok(dev_configuration) => dev_configuration,
                Err(error) => {
                    tracing::error!("{}", error.to_string().trim());
                    continue;
                }
            };

            if let Err(err) = config_sender.send(dev_configuration.merged_configuration.clone()) {
                tracing::error!("Could not update config: {err}");
                continue;
            };

            let merged_configuration = Arc::new(dev_configuration.merged_configuration);
            let overridden_subgraphs = Arc::new(dev_configuration.overridden_subgraphs);

            if let Err(error) = reload_subgraphs(
                sdl_sender.clone(),
                subgraph_cache.clone(),
                overridden_subgraphs.clone(),
                merged_configuration.clone(),
                graph_overrides_path,
                None,
            )
            .await
            {
                tracing::error!("{}", error.to_string().trim());
                continue;
            }

            tracing::info!("detected a configuration change, reloading");

            let _ = subgraph_watcher
                .start(
                    sdl_sender.clone(),
                    subgraph_cache,
                    overridden_subgraphs,
                    merged_configuration,
                    graph_overrides_path,
                )
                .inspect_err(|error| tracing::error!("{}", error.to_string().trim()));
        }

        Ok::<_, BackendError>(())
    });
}

async fn reload_subgraphs(
    sender: mpsc::Sender<String>,
    subgraph_cache: Arc<SubgraphCache>,
    overridden_subgraphs: Arc<HashSet<String>>,
    merged_configuration: Arc<Config>,
    graph_overrides_path: Option<&'static PathBuf>,
    cancellation_token: Option<CancellationToken>,
) -> Result<(), BackendError> {
    let mut subgraphs = graphql_composition::Subgraphs::default();

    for (name, remote_subgraph) in subgraph_cache
        .remote
        .iter()
        .filter(|(name, _)| !overridden_subgraphs.contains(**name))
    {
        subgraphs
            .ingest_str(&remote_subgraph.schema, name, &remote_subgraph.url)
            .map_err(BackendError::IngestSubgraph)?;
    }

    // we're not passing in the graph ref to avoid fetching the remote subgraphs again
    // as we have them cached
    get_subgraph_sdls(
        None,
        &overridden_subgraphs,
        &merged_configuration,
        &mut subgraphs,
        graph_overrides_path,
    )
    .await?;

    let composition_result = graphql_composition::compose(&subgraphs);

    let federated_sdl = match composition_result.into_result() {
        Ok(result) => federated_graph::render_federated_sdl(&result).map_err(BackendError::ToFederatedSdl)?,
        Err(diagnostics) => {
            return Err(BackendError::Composition(
                diagnostics.iter_messages().collect::<Vec<_>>().join("\n"),
            ));
        }
    };

    // recheck cancelation right before sending to reduce the window
    // where a configuration reload start could overlap a subgraph watcher reload.
    // this prevents most cases of a subgraph reload going through only to immediatelly follow a config reload
    // (although even if that happens we'll still have valid state due to this happening first)
    if cancellation_token.is_some_and(|token| token.is_cancelled()) {
        return Ok(());
    }

    let _ = sender.send(federated_sdl).await;

    Ok(())
}
