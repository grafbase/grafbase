use fxhash::FxHashMap;
use gateway_config::{SubgraphConfig, SubscriptionProtocol};

use super::{
    BuildError, Context, ExtensionDirectiveId, GraphContext, GraphqlEndpointId, GraphqlEndpointRecord, SubgraphId,
    VirtualSubgraphId, VirtualSubgraphRecord,
};

#[derive(Default, id_derives::IndexedFields)]
pub(crate) struct SubgraphsContext {
    pub all: Vec<SubgraphId>,
    pub id_mapping: FxHashMap<federated_graph::SubgraphId, SubgraphId>,
    #[indexed_by(GraphqlEndpointId)]
    pub graphql_endpoints: Vec<GraphqlEndpointRecord>,
    #[indexed_by(VirtualSubgraphId)]
    pub virtual_subgraphs: Vec<VirtualSubgraphRecord>,
}

impl std::ops::Index<federated_graph::SubgraphId> for SubgraphsContext {
    type Output = SubgraphId;
    fn index(&self, id: federated_graph::SubgraphId) -> &Self::Output {
        &self.id_mapping[&id]
    }
}

impl Context<'_> {
    pub(super) fn load_subgraphs(&mut self) -> Result<(), BuildError> {
        let mut subgraphs = SubgraphsContext {
            all: Vec::with_capacity(self.federated_graph.subgraphs.len()),
            id_mapping: FxHashMap::with_capacity_and_hasher(self.federated_graph.subgraphs.len(), Default::default()),
            graphql_endpoints: Vec::new(),
            virtual_subgraphs: Vec::new(),
        };

        let default_cache_ttl = if self.config.entity_caching.enabled {
            Some(self.config.entity_caching.ttl)
        } else {
            None
        };

        for (ix, subgraph) in self.federated_graph.subgraphs.iter().enumerate() {
            let id = federated_graph::SubgraphId::from(ix);
            let subgraph_name_id = self.get_or_insert_str(subgraph.name);
            let SubgraphConfig {
                url,
                headers,
                websocket_url,
                timeout,
                retry,
                entity_caching,
                subscription_protocol,
                ..
            } = self
                .config
                .subgraphs
                .get(&self.federated_graph[subgraph.name])
                .cloned()
                .unwrap_or_default();

            let url = url
                .map(Ok)
                .or_else(|| {
                    subgraph.url.map(|url| {
                        let url = &self.federated_graph[url];
                        url::Url::parse(url).map_err(|err| BuildError::InvalidUrl {
                            url: url.to_string(),
                            err: err.to_string(),
                        })
                    })
                })
                .transpose()?;

            let subgraph_id = if let Some(url) = url {
                subgraphs.graphql_endpoints.push(GraphqlEndpointRecord {
                    subgraph_name_id,
                    url_id: self.urls.insert(url),
                    subscription_protocol: match subscription_protocol {
                        Some(protocol) => protocol,
                        None if websocket_url.is_some() => SubscriptionProtocol::Websocket,
                        None => SubscriptionProtocol::ServerSentEvents,
                    },
                    websocket_url_id: websocket_url.clone().map(|url| self.urls.insert(url)),
                    header_rule_ids: self.ingest_header_rules(&headers),
                    config: super::SubgraphConfig {
                        timeout,
                        retry: retry.map(Into::into),
                        cache_ttl: entity_caching
                            .as_ref()
                            .and_then(|cfg| {
                                cfg.enabled
                                    .unwrap_or(self.config.entity_caching.enabled)
                                    .then_some(cfg.ttl)
                                    .flatten()
                            })
                            .or(default_cache_ttl),
                    },
                    schema_directive_ids: Vec::new(),
                });
                SubgraphId::GraphqlEndpoint((subgraphs.graphql_endpoints.len() - 1).into())
            } else {
                subgraphs.virtual_subgraphs.push(VirtualSubgraphRecord {
                    subgraph_name_id,
                    schema_directive_ids: Vec::new(),
                });
                SubgraphId::Virtual((subgraphs.virtual_subgraphs.len() - 1).into())
            };
            subgraphs.all.push(subgraph_id);
            subgraphs.id_mapping.insert(id, subgraph_id);
        }

        self.subgraphs = subgraphs;

        Ok(())
    }
}

impl GraphContext<'_> {
    pub(super) fn push_extension_schema_directive(&mut self, id: ExtensionDirectiveId) {
        let subgraph_id = self.graph[id].subgraph_id;
        match subgraph_id {
            SubgraphId::GraphqlEndpoint(subgraph_id) => {
                self.subgraphs[subgraph_id].schema_directive_ids.push(id.into());
            }
            SubgraphId::Virtual(subgraph_id) => {
                self.subgraphs[subgraph_id].schema_directive_ids.push(id.into());
            }
            SubgraphId::Introspection => unreachable!(),
        }
    }
}
