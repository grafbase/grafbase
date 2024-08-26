use std::{mem::take, time::Duration};

use config::latest::Config;
use federated_graph::FederatedGraph;

use super::{
    sources::{
        graphql::{GraphqlEndpoint, RetryConfig},
        GraphqlEndpoints,
    },
    BuildContext,
};

pub struct ExternalDataSources {
    pub graphql: GraphqlEndpoints,
}

impl ExternalDataSources {
    pub(super) fn build(ctx: &mut BuildContext, config: &mut Config, graph: &mut FederatedGraph) -> Self {
        let endpoints = take(&mut graph.subgraphs)
            .into_iter()
            .enumerate()
            .map(|(index, subgraph)| {
                let subgraph_id = ctx.next_subgraph_id();
                let name = subgraph.name.into();
                let url = ctx
                    .urls
                    .insert(url::Url::parse(&ctx.strings[subgraph.url.into()]).expect("valid url"));
                match config.subgraph_configs.remove(&federated_graph::SubgraphId(index)) {
                    Some(config::latest::SubgraphConfig {
                        websocket_url,
                        headers,
                        timeout,
                        retry,
                        entity_caching,
                        ..
                    }) => GraphqlEndpoint {
                        subgraph_name: name,
                        subgraph_id,
                        url,
                        websocket_url: websocket_url
                            .map(|url| ctx.urls.insert(url::Url::parse(&config[url]).expect("valid url"))),
                        header_rules: headers.into_iter().map(Into::into).collect(),
                        timeout: timeout.unwrap_or(DEFAULT_SUBGRAPH_TIMEOUT),
                        retry: retry.map(
                            |config::latest::RetryConfig {
                                 min_per_second,
                                 ttl,
                                 retry_percent,
                                 retry_mutations,
                             }| RetryConfig {
                                min_per_second,
                                ttl,
                                retry_percent,
                                retry_mutations,
                            },
                        ),
                        entity_cache_ttl: entity_caching.as_ref().unwrap_or(&config.entity_caching).ttl(),
                    },

                    None => GraphqlEndpoint {
                        subgraph_name: name,
                        subgraph_id,
                        url,
                        websocket_url: None,
                        header_rules: Vec::new(),
                        timeout: DEFAULT_SUBGRAPH_TIMEOUT,
                        retry: None,
                        entity_cache_ttl: config.entity_caching.ttl(),
                    },
                }
            })
            .collect();
        ExternalDataSources {
            graphql: GraphqlEndpoints { endpoints },
        }
    }
}

const DEFAULT_SUBGRAPH_TIMEOUT: Duration = Duration::from_secs(30);
