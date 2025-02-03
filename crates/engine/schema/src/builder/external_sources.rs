use std::{mem::take, time::Duration};

use federated_graph::FederatedGraph;
use fxhash::FxHashMap;
use gateway_config::{SubgraphConfig, SubscriptionProtocol};

use super::{
    BuildContext, BuildError, GraphqlEndpointId, GraphqlEndpointRecord, SubgraphId, VirtualSubgraphId,
    VirtualSubgraphRecord,
};

#[derive(id_derives::IndexedFields)]
pub struct ExternalDataSources {
    pub id_mapping: FxHashMap<federated_graph::SubgraphId, SubgraphId>,
    #[indexed_by(GraphqlEndpointId)]
    pub graphql_endpoints: Vec<GraphqlEndpointRecord>,
    #[indexed_by(VirtualSubgraphId)]
    pub virtual_subgraphs: Vec<VirtualSubgraphRecord>,
}

impl std::ops::Index<federated_graph::SubgraphId> for ExternalDataSources {
    type Output = SubgraphId;
    fn index(&self, id: federated_graph::SubgraphId) -> &Self::Output {
        &self.id_mapping[&id]
    }
}

impl ExternalDataSources {
    pub(super) fn build(ctx: &mut BuildContext<'_>, graph: &mut FederatedGraph) -> Result<Self, BuildError> {
        let mut sources = ExternalDataSources {
            id_mapping: FxHashMap::with_capacity_and_hasher(graph.subgraphs.len(), Default::default()),
            graphql_endpoints: Vec::new(),
            virtual_subgraphs: Vec::new(),
        };

        let default_cache_ttl = if ctx.config.entity_caching.enabled {
            Some(ctx.config.entity_caching.ttl)
        } else {
            None
        };

        for (index, subgraph) in take(&mut graph.subgraphs).into_iter().enumerate() {
            let subgraph_name_id = ctx.strings.get_or_new(&graph[subgraph.name]);
            let id = federated_graph::SubgraphId::from(index);
            let config = ctx.config.subgraphs.get(&graph[subgraph.name]);

            if subgraph.url.is_some() || config.is_some_and(|cfg| cfg.url.is_some()) {
                match config {
                    Some(SubgraphConfig {
                        subscription_protocol,
                        websocket_url,
                        url,
                        headers,
                        timeout,
                        retry,
                        entity_caching,
                        ..
                    }) => sources.graphql_endpoints.push(GraphqlEndpointRecord {
                        subgraph_name_id,
                        url_id: ctx.urls.insert(match url {
                            Some(url) => url.clone(),
                            None => {
                                let url = &graph[subgraph.url.expect("Config doesn't exist, so SDL must have URL")];
                                url::Url::parse(url).map_err(|err| BuildError::InvalidUrl {
                                    url: url.to_string(),
                                    err: err.to_string(),
                                })?
                            }
                        }),
                        subscription_protocol: match subscription_protocol {
                            Some(protocol) => *protocol,
                            None if websocket_url.is_some() => SubscriptionProtocol::Websocket,
                            None => SubscriptionProtocol::ServerSentEvents,
                        },

                        websocket_url_id: websocket_url.clone().map(|url| ctx.urls.insert(url)),
                        header_rule_ids: ctx.ingest_header_rules(headers),
                        config: super::SubgraphConfig {
                            timeout: timeout.unwrap_or(DEFAULT_SUBGRAPH_TIMEOUT),
                            retry: retry.map(Into::into),
                            cache_ttl: entity_caching
                                .as_ref()
                                .and_then(|cfg| {
                                    cfg.enabled
                                        .unwrap_or(ctx.config.entity_caching.enabled)
                                        .then_some(cfg.ttl)
                                        .flatten()
                                })
                                .or(default_cache_ttl),
                        },
                        schema_directive_ids: Vec::new(),
                    }),
                    None => sources.graphql_endpoints.push(GraphqlEndpointRecord {
                        subgraph_name_id,
                        url_id: {
                            let url = &graph[subgraph.url.expect("Config doesn't exist, so SDL must have URL")];
                            let url = url::Url::parse(url).map_err(|err| BuildError::InvalidUrl {
                                url: url.to_string(),
                                err: err.to_string(),
                            })?;
                            ctx.urls.insert(url)
                        },
                        websocket_url_id: None,
                        subscription_protocol: SubscriptionProtocol::ServerSentEvents,
                        header_rule_ids: Default::default(),
                        config: super::SubgraphConfig {
                            timeout: DEFAULT_SUBGRAPH_TIMEOUT,
                            retry: None,
                            cache_ttl: default_cache_ttl,
                        },
                        schema_directive_ids: Vec::new(),
                    }),
                }
                sources.id_mapping.insert(
                    id,
                    SubgraphId::GraphqlEndpoint((sources.graphql_endpoints.len() - 1).into()),
                );
            } else {
                sources.virtual_subgraphs.push(VirtualSubgraphRecord {
                    subgraph_name_id,
                    schema_directive_ids: Vec::new(),
                });
                sources
                    .id_mapping
                    .insert(id, SubgraphId::Virtual((sources.virtual_subgraphs.len() - 1).into()));
            }
        }

        Ok(sources)
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = SubgraphId> + '_ {
        self.id_mapping.values().copied()
    }
}

const DEFAULT_SUBGRAPH_TIMEOUT: Duration = Duration::from_secs(30);
