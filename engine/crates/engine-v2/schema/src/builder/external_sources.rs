use std::{mem::take, time::Duration};

use config::latest::Config;

use super::{BuildContext, GraphqlEndpointRecord};

pub struct ExternalDataSources {
    pub graphql_endpoints: Vec<GraphqlEndpointRecord>,
}

impl ExternalDataSources {
    pub(super) fn build(ctx: &mut BuildContext, config: &mut Config) -> Self {
        let graphql_endpoints = take(&mut config.graph.subgraphs)
            .into_iter()
            .enumerate()
            .map(|(index, subgraph)| {
                let subgraph_name_id = subgraph.name.into();
                let sdl_url = url::Url::parse(&ctx.strings[subgraph.url.into()]).expect("valid url");
                match config.subgraph_configs.remove(&federated_graph::SubgraphId(index)) {
                    Some(config::latest::SubgraphConfig {
                        websocket_url,
                        url,
                        headers,
                        timeout,
                        retry,
                        entity_caching,
                        ..
                    }) => GraphqlEndpointRecord {
                        subgraph_name_id,
                        url_id: ctx.urls.insert(url.unwrap_or(sdl_url)),
                        websocket_url_id: websocket_url
                            .map(|url| ctx.urls.insert(url::Url::parse(&config[url]).expect("valid url"))),
                        header_rule_ids: headers.into_iter().map(Into::into).collect(),
                        config: super::SubgraphConfig {
                            timeout: timeout.unwrap_or(DEFAULT_SUBGRAPH_TIMEOUT),
                            retry: retry.map(Into::into),
                            cache_ttl: entity_caching.as_ref().unwrap_or(&config.entity_caching).ttl(),
                        },
                    },

                    None => GraphqlEndpointRecord {
                        subgraph_name_id,
                        url_id: ctx.urls.insert(sdl_url),
                        websocket_url_id: None,
                        header_rule_ids: Vec::new(),
                        config: super::SubgraphConfig {
                            timeout: DEFAULT_SUBGRAPH_TIMEOUT,
                            retry: None,
                            cache_ttl: config.entity_caching.ttl(),
                        },
                    },
                }
            })
            .collect();
        ExternalDataSources { graphql_endpoints }
    }
}

const DEFAULT_SUBGRAPH_TIMEOUT: Duration = Duration::from_secs(30);
