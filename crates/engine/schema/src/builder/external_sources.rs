use std::{mem::take, time::Duration};

use config::{Config, SubgraphConfig};
use fxhash::FxHashMap;
use gateway_config::SubscriptionProtocol;

use super::{
    BuildContext, GraphqlEndpointId, GraphqlEndpointRecord, SubgraphId, VirtualSubgraphId, VirtualSubgraphRecord,
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
    pub(super) fn build(ctx: &mut BuildContext<'_>, config: &mut Config) -> Self {
        let mut sources = ExternalDataSources {
            id_mapping: Default::default(),
            graphql_endpoints: Vec::new(),
            virtual_subgraphs: Vec::new(),
        };

        for (index, subgraph) in take(&mut config.graph.subgraphs).into_iter().enumerate() {
            let subgraph_name_id = subgraph.name.into();
            let id = federated_graph::SubgraphId::from(index);
            if let Some(url) = subgraph.url {
                let sdl_url = url::Url::parse(&ctx.strings[url.into()]).expect("valid url");
                match config.subgraph_configs.remove(&id) {
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
                        url_id: ctx.urls.insert(url.unwrap_or(sdl_url)),
                        subscription_protocol: match subscription_protocol {
                            Some(protocol) => protocol,
                            None if websocket_url.is_some() => SubscriptionProtocol::Websocket,
                            None => SubscriptionProtocol::ServerSentEvents,
                        },

                        websocket_url_id: websocket_url
                            .map(|url| ctx.urls.insert(url::Url::parse(&config[url]).expect("valid url"))),
                        header_rule_ids: headers.into_iter().map(Into::into).collect(),
                        config: super::SubgraphConfig {
                            timeout: timeout.unwrap_or(DEFAULT_SUBGRAPH_TIMEOUT),
                            retry: retry.map(Into::into),
                            cache_ttl: entity_caching.as_ref().unwrap_or(&config.entity_caching).ttl(),
                        },
                        schema_directive_ids: Vec::new(),
                    }),
                    None => sources.graphql_endpoints.push(GraphqlEndpointRecord {
                        subgraph_name_id,
                        url_id: ctx.urls.insert(sdl_url),
                        websocket_url_id: None,
                        subscription_protocol: SubscriptionProtocol::ServerSentEvents,
                        header_rule_ids: Vec::new(),
                        config: super::SubgraphConfig {
                            timeout: DEFAULT_SUBGRAPH_TIMEOUT,
                            retry: None,
                            cache_ttl: config.entity_caching.ttl(),
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

        sources
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = SubgraphId> + '_ {
        self.id_mapping.values().copied()
    }
}

const DEFAULT_SUBGRAPH_TIMEOUT: Duration = Duration::from_secs(30);
