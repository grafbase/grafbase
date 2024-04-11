use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use futures_util::StreamExt;
use parser_sdl::federation::{SubgraphConfig, SubgraphHeaderValue};
use tokio::task::JoinSet;
use tokio_stream::wrappers::WatchStream;
use url::Url;

use crate::{dev::admin::Header, events::emit_event, ConfigWatcher, FederatedDevEvent};

use super::bus::SubgraphConfigWatcherBus;

/// Watches the config and updates `Composer` based on any hardcoded subgraph details
/// in the config
pub(crate) struct SubgraphConfigWatcher {
    config: ConfigWatcher,
    bus: SubgraphConfigWatcherBus,
}

impl SubgraphConfigWatcher {
    pub(crate) fn new(config: ConfigWatcher, bus: SubgraphConfigWatcherBus) -> Self {
        SubgraphConfigWatcher { config, bus }
    }

    #[tracing::instrument(skip_all)]
    pub async fn handler(self) {
        let SubgraphConfigWatcher { config, bus } = self;

        let mut retry_tasks = JoinSet::new();

        let mut config_stream = WatchStream::new(config);

        let mut previous_subgraphs = BTreeMap::new();

        while let Some(next_config) = config_stream.next().await {
            // Cancel any running retry tasks
            retry_tasks.shutdown().await;

            // We can only really instantiate subgraphs for which we have a URL
            let next_subgraphs = next_config
                .subgraphs
                .into_iter()
                .filter(|(_, subgraph)| subgraph.development_url.is_some())
                .collect();

            let changes = determine_changes(&previous_subgraphs, &next_subgraphs);

            tracing::debug!("applying changed subgraph configs: {changes:?}");

            for config in changes.new_subgraphs.into_iter().chain(changes.changed_subgraphs) {
                let Ok(url) = config.development_url.as_ref().unwrap().parse::<Url>() else {
                    continue;
                };

                let headers = config
                    .headers
                    .iter()
                    .filter_map(|(key, value)| match value {
                        SubgraphHeaderValue::Static(value) => Some(Header {
                            key: key.clone(),
                            value: value.clone(),
                        }),
                        SubgraphHeaderValue::Forward(_) => None,
                    })
                    .collect::<Vec<_>>();

                let schema = match bus.introspect_schema(&config.name, url.clone(), headers.clone()).await {
                    Ok(schema) => schema,
                    Err(error) => {
                        // Log the error once and then start up a task that'll silently retry in the background
                        emit_event(FederatedDevEvent::PredefinedIntrospectionFailed {
                            subgraph_name: config.name.clone(),
                            rendered_error: error.to_string(),
                        });
                        retry_tasks.spawn(retry_subgraph(bus.clone(), config.name.clone(), url, headers));
                        continue;
                    }
                };

                bus.compose_graph(config.name.clone(), url, headers, schema).await.ok();
            }

            for config in changes.deleted_subgraphs {
                bus.remove_subgraph(&config.name).await.ok();
            }

            previous_subgraphs = next_subgraphs;
        }
    }
}

#[tracing::instrument(skip(bus))]
async fn retry_subgraph(bus: SubgraphConfigWatcherBus, name: String, url: Url, headers: Vec<Header>) {
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let Ok(schema) = bus.introspect_schema(&name, url.clone(), headers.clone()).await else {
            tracing::debug!("introspection retry failed");
            continue;
        };

        if bus
            .compose_graph(name.clone(), url.clone(), headers.clone(), schema)
            .await
            .is_ok()
        {
            break;
        }
        tracing::debug!("composition retry failed");
    }
}

fn determine_changes<'a>(
    previous_subgraphs: &'a BTreeMap<String, SubgraphConfig>,
    next_subgraphs: &'a BTreeMap<String, SubgraphConfig>,
) -> ConfigChanges<'a> {
    let previous_names = previous_subgraphs.keys().collect::<BTreeSet<_>>();
    let next_names = next_subgraphs.keys().collect::<BTreeSet<_>>();

    let new_names = next_names.difference(&previous_names).collect::<BTreeSet<_>>();
    let deleted_names = previous_names.difference(&next_names).collect::<BTreeSet<_>>();

    let changed_subgraphs = {
        let previous_subgraphs = previous_subgraphs
            .iter()
            .filter(|(name, _)| !deleted_names.contains(name))
            .collect::<BTreeSet<_>>();

        let next_subgraphs = next_subgraphs
            .iter()
            .filter(|(name, _)| !new_names.contains(name))
            .collect::<BTreeSet<_>>();

        next_subgraphs
            .difference(&previous_subgraphs)
            .map(|(_, config)| *config)
            .collect::<Vec<_>>()
    };

    let new_subgraphs = new_names.into_iter().map(|name| &next_subgraphs[*name]).collect();
    let deleted_subgraphs = deleted_names
        .into_iter()
        .map(|name| &previous_subgraphs[*name])
        .collect();

    ConfigChanges {
        new_subgraphs,
        deleted_subgraphs,
        changed_subgraphs,
    }
}

#[derive(Debug)]
struct ConfigChanges<'a> {
    new_subgraphs: Vec<&'a SubgraphConfig>,
    deleted_subgraphs: Vec<&'a SubgraphConfig>,
    changed_subgraphs: Vec<&'a SubgraphConfig>,
}
