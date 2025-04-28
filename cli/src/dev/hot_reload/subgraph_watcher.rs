use super::{super::subgraphs::SubgraphCache, WATCHER_DEBOUNCE_DURATION, reload_subgraphs};
use crate::{
    dev::subgraphs::{CachedIntrospectedSubgraph, CachedSubgraph},
    errors::BackendError,
};
use gateway_config::Config;
use grafbase_graphql_introspection::introspect;
use notify_debouncer_full::{
    DebounceEventResult, Debouncer, RecommendedCache, new_debouncer,
    notify::{EventKind, RecommendedWatcher, RecursiveMode},
};
use std::{collections::HashSet, sync::Arc};
use tokio::{runtime::Handle, sync::mpsc, time::MissedTickBehavior};
use tokio_util::sync::CancellationToken;

pub(super) struct SubgraphWatcher {
    watcher: Option<Debouncer<RecommendedWatcher, RecommendedCache>>,
    cancellation_token: Option<CancellationToken>,
}

impl SubgraphWatcher {
    pub(super) fn new() -> Self {
        Self {
            watcher: None,
            cancellation_token: None,
        }
    }

    pub(super) fn stop(&mut self) {
        self.watcher = None;
        if let Some(ref poller_cancellation_token) = self.cancellation_token {
            poller_cancellation_token.cancel();
        }
        self.cancellation_token = None;
    }

    pub(super) fn start(
        &mut self,
        sender: mpsc::Sender<String>,
        subgraph_cache: Arc<SubgraphCache>,
        overridden_subgraphs: Arc<HashSet<String>>,
        config: Arc<Config>,
    ) -> Result<(), BackendError> {
        // skip if there's no local subgraphs
        if overridden_subgraphs.is_empty() {
            return Ok(());
        }

        self.cancellation_token = Some(CancellationToken::new());

        self.spawn_introspection_poller(sender.clone(), subgraph_cache.clone(), config.clone())?;

        self.spawn_schema_file_watcher(sender, subgraph_cache, config)
    }

    fn spawn_introspection_poller(
        &mut self,
        sender: mpsc::Sender<String>,
        subgraph_cache: Arc<SubgraphCache>,
        config: Arc<Config>,
    ) -> Result<(), BackendError> {
        let introspection_urls = config
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

                let mut cached_local_subgraphs = subgraph_cache.local_from_introspection.lock().await;

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
                        let changed = sdl != cached_local_subgraph.subgraph.sdl;
                        if changed {
                            Ok::<_, BackendError>(Some((
                                name.clone(),
                                CachedIntrospectedSubgraph {
                                    subgraph: Arc::new(CachedSubgraph {
                                        sdl,
                                        name: cached_local_subgraph.subgraph.name.clone(),
                                        url: cached_local_subgraph.subgraph.url.clone(),
                                    }),
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
                        config.clone(),
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

    fn spawn_schema_file_watcher(
        &mut self,
        sender: mpsc::Sender<String>,
        subgraph_cache: Arc<SubgraphCache>,
        config: Arc<Config>,
    ) -> Result<(), BackendError> {
        let schema_file_paths = config
            .subgraphs
            .iter()
            .filter_map(|(_name, subgraph)| subgraph.schema_path.as_ref())
            .collect::<Vec<_>>();

        if schema_file_paths.is_empty() {
            return Ok(());
        }

        let runtime_handle = Handle::current();

        let watcher_cancellation_token = self.cancellation_token.as_ref().expect("must exist").child_token();

        let watcher_merged_configuration = config.clone();

        let mut watcher = new_debouncer(WATCHER_DEBOUNCE_DURATION, None, move |result: DebounceEventResult| {
            let Ok(result) = result else {
                return;
            };

            assert!(!result.is_empty());

            let subgraph_cache = subgraph_cache.clone();
            let config = watcher_merged_configuration.clone();
            let sender = sender.clone();

            if watcher_cancellation_token.is_cancelled() {
                return;
            }

            let should_reload = result.iter().any(|event| {
                matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                )
            });

            if !should_reload {
                return;
            }

            let reload_cancellation_token = watcher_cancellation_token.child_token();

            tracing::info!("Detected a subgraph schema file change, reloading.",);

            runtime_handle.block_on(async move {
                match reload_subgraphs(sender.clone(), subgraph_cache, config, Some(reload_cancellation_token)).await {
                    Ok(()) => tracing::debug!("Subgraphs reload successful"),
                    Err(error) => tracing::error!("Error reloading subgraphs: {}", error.to_string().trim()),
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
