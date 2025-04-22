mod subgraph_watcher;

use self::subgraph_watcher::*;
use super::subgraphs::SubgraphCache;
use crate::errors::BackendError;
use gateway_config::Config;
use notify_debouncer_full::{
    DebounceEventResult, new_debouncer,
    notify::{self, RecursiveMode},
};
use std::{collections::HashSet, path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    runtime::Handle,
    sync::{broadcast::Receiver, mpsc, watch},
};
use tokio_stream::{StreamExt, wrappers::ReceiverStream};
use tokio_util::sync::CancellationToken;

const WATCHER_DEBOUNCE_DURATION: Duration = Duration::from_secs(2);

fn watch_configuration_files(gateway_config_path: Option<&PathBuf>) -> Result<mpsc::Receiver<()>, notify::Error> {
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

    // since the config watcher should live for the remainder of the cli run,
    // leak it instead of needing to make sure it isn't dropped
    Box::leak(Box::new(watcher));

    Ok(watcher_receiver)
}

pub(crate) async fn hot_reload(
    config_sender: watch::Sender<Config>,
    sdl_sender: mpsc::Sender<String>,
    mut ready_receiver: Receiver<String>,
    composition_warnings_sender: mpsc::Sender<Vec<String>>,
    subgraph_cache: Arc<SubgraphCache>,
    config: Config,
) {
    // start hot reloading once the server is ready
    if ready_receiver.recv().await.is_err() {
        return;
    }

    let Some(config_path) = config.path.clone() else {
        // return early since we don't hot reload graphs from the API
        return;
    };

    let Ok(watcher_receiver) = watch_configuration_files(Some(&config_path))
        .map_err(BackendError::SetUpWatcher)
        .inspect_err(|error| tracing::error!("{}", error.to_string().trim()))
    else {
        return;
    };

    let mut subgraph_watcher = SubgraphWatcher::new();

    let overridden_subgraphs = Arc::new(
        config
            .subgraphs
            .iter()
            .filter_map(|(name, subgraph)| {
                if subgraph.has_schema_override() {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>(),
    );

    let config = Arc::new(config);

    // don't skip the config reloader if
    // the subgraph watcher encountered an error
    let _ = subgraph_watcher
        .start(
            sdl_sender.clone(),
            composition_warnings_sender.clone(),
            subgraph_cache.clone(),
            overridden_subgraphs.clone(),
            config,
        )
        .inspect_err(|error| tracing::error!("{}", error.to_string().trim()));

    let mut stream = ReceiverStream::new(watcher_receiver);
    while stream.next().await.is_some() {
        let subgraph_cache = subgraph_cache.clone();
        subgraph_watcher.stop();

        let config = match Config::load(&config_path) {
            Ok(config) => config,
            Err(error) => {
                tracing::error!("{}", error.to_string().trim());
                continue;
            }
        };

        if let Err(err) = config_sender.send(config.clone()) {
            tracing::error!("Could not update config: {err}");
            continue;
        };

        let config = Arc::new(config);

        if let Err(error) = reload_subgraphs(
            sdl_sender.clone(),
            composition_warnings_sender.clone(),
            subgraph_cache.clone(),
            config.clone(),
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
                composition_warnings_sender.clone(),
                subgraph_cache,
                overridden_subgraphs.clone(),
                config,
            )
            .inspect_err(|error| tracing::error!("{}", error.to_string().trim()));
    }
}

async fn reload_subgraphs(
    sender: mpsc::Sender<String>,
    composition_warnings_sender: mpsc::Sender<Vec<String>>,
    subgraph_cache: Arc<SubgraphCache>,
    config: Arc<Config>,
    cancellation_token: Option<CancellationToken>,
) -> Result<(), BackendError> {
    subgraph_cache.reload_local_subgraphs(&config).await?;
    let composition_result = subgraph_cache.compose().await?;

    {
        let mut warnings = composition_result.diagnostics().iter_warnings().peekable();

        if warnings.peek().is_some() {
            composition_warnings_sender
                .send(warnings.map(ToOwned::to_owned).collect())
                .await
                .unwrap();
        }
    }

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
