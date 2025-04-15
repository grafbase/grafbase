use gateway_config::{SubgraphConfig, SubscriptionProtocol};
use id_newtypes::IdRange;
use rapidhash::RapidHashMap;

use crate::{
    ForwardHeaderRuleRecord, HeaderRuleId, HeaderRuleRecord, InsertHeaderRuleRecord, NameOrPatternId,
    RemoveHeaderRuleRecord, RenameDuplicateHeaderRuleRecord, SubGraphs, introspection::IntrospectionSubgraph,
};

use super::{
    GraphqlEndpointId, GraphqlEndpointRecord, SubgraphId, VirtualSubgraphId, VirtualSubgraphRecord,
    context::Interners,
    error::Error,
    sdl::{self, GraphName, Sdl},
};

#[derive(Default, id_derives::IndexedFields)]
pub(crate) struct SubgraphsBuilder<'sdl> {
    pub all: Vec<SubgraphId>,
    pub mapping: RapidHashMap<GraphName<'sdl>, SubgraphId>,
    #[indexed_by(GraphqlEndpointId)]
    pub graphql_endpoints: Vec<GraphqlEndpointRecord>,
    #[indexed_by(VirtualSubgraphId)]
    pub virtual_subgraphs: Vec<VirtualSubgraphRecord>,
    pub default_header_rules: IdRange<HeaderRuleId>,
    pub header_rules: Vec<HeaderRuleRecord>,
}

impl<'sdl> SubgraphsBuilder<'sdl> {
    pub(super) fn new(sdl: &'sdl Sdl<'sdl>, config: &gateway_config::Config, interners: &mut Interners) -> Self {
        let mut subgraphs = SubgraphsBuilder {
            all: Vec::with_capacity(sdl.subgraphs.len()),
            mapping: RapidHashMap::with_capacity_and_hasher(sdl.subgraphs.len(), Default::default()),
            graphql_endpoints: Vec::new(),
            virtual_subgraphs: Vec::new(),
            header_rules: Vec::new(),
            default_header_rules: IdRange::default(),
        };

        subgraphs.default_header_rules = ingest_header_rules(&mut subgraphs.header_rules, &config.headers, interners);

        let default_cache_ttl = if config.entity_caching.enabled {
            Some(config.entity_caching.ttl)
        } else {
            None
        };

        for (&graph_enum_name, subgraph) in &sdl.subgraphs {
            let name = subgraph.name.unwrap_or(graph_enum_name.as_str());
            let subgraph_name_id = interners.strings.get_or_new(name);
            let SubgraphConfig {
                url,
                headers,
                websocket_url,
                timeout,
                retry,
                entity_caching,
                subscription_protocol,
                ..
            } = config.subgraphs.get(name).cloned().unwrap_or_default();
            let url = url.or(subgraph.url.clone());

            let header_rule_ids = ingest_header_rules(&mut subgraphs.header_rules, &headers, interners);
            let subgraph_id = if let Some(url) = url {
                subgraphs.graphql_endpoints.push(GraphqlEndpointRecord {
                    subgraph_name_id,
                    url_id: interners.urls.insert(url),
                    subscription_protocol: match subscription_protocol {
                        Some(protocol) => protocol,
                        None if websocket_url.is_some() => SubscriptionProtocol::Websocket,
                        None => SubscriptionProtocol::ServerSentEvents,
                    },
                    websocket_url_id: websocket_url.clone().map(|url| interners.urls.insert(url)),
                    header_rule_ids,
                    config: super::SubgraphConfig {
                        timeout,
                        retry: retry.map(Into::into),
                        cache_ttl: entity_caching
                            .as_ref()
                            .and_then(|cfg| {
                                cfg.enabled
                                    .unwrap_or(config.entity_caching.enabled)
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
                    header_rule_ids,
                });
                SubgraphId::Virtual((subgraphs.virtual_subgraphs.len() - 1).into())
            };
            subgraphs.all.push(subgraph_id);
            subgraphs.mapping.insert(graph_enum_name, subgraph_id);
        }

        subgraphs
    }

    pub(super) fn try_get(&self, name: GraphName<'_>, span: sdl::Span) -> Result<SubgraphId, Error> {
        self.mapping
            .get(&name)
            .copied()
            .ok_or_else(|| (format!("Graph named '{name}' does not exist."), span).into())
    }

    pub(super) fn finalize_with(self, introspection: IntrospectionSubgraph) -> SubGraphs {
        let Self {
            graphql_endpoints,
            virtual_subgraphs,
            default_header_rules,
            header_rules,
            ..
        } = self;
        SubGraphs {
            graphql_endpoints,
            virtual_subgraphs,
            introspection,
            default_header_rules,
            header_rules,
        }
    }
}

fn ingest_header_rules(
    header_rules: &mut Vec<HeaderRuleRecord>,
    rules: &[gateway_config::HeaderRule],
    interners: &mut Interners,
) -> IdRange<HeaderRuleId> {
    use gateway_config::*;
    let start = header_rules.len();
    header_rules.extend(rules.iter().map(|rule| -> HeaderRuleRecord {
        match rule {
            HeaderRule::Forward(rule) => {
                let name_id = match &rule.name {
                    NameOrPattern::Pattern(regex) => {
                        NameOrPatternId::Pattern(interners.regexps.get_or_insert(regex.clone()))
                    }
                    NameOrPattern::Name(name) => NameOrPatternId::Name(interners.strings.get_or_new(name.as_ref())),
                };

                let default_id = rule.default.as_ref().map(|s| interners.strings.get_or_new(s.as_ref()));
                let rename_id = rule.rename.as_ref().map(|s| interners.strings.get_or_new(s.as_ref()));

                HeaderRuleRecord::Forward(ForwardHeaderRuleRecord {
                    name_id,
                    default_id,
                    rename_id,
                })
            }
            HeaderRule::Insert(rule) => {
                let name_id = interners.strings.get_or_new(rule.name.as_ref());
                let value_id = interners.strings.get_or_new(rule.value.as_ref());

                HeaderRuleRecord::Insert(InsertHeaderRuleRecord { name_id, value_id })
            }
            HeaderRule::Remove(rule) => {
                let name_id = match &rule.name {
                    NameOrPattern::Pattern(regex) => {
                        NameOrPatternId::Pattern(interners.regexps.get_or_insert(regex.clone()))
                    }
                    NameOrPattern::Name(name) => NameOrPatternId::Name(interners.strings.get_or_new(name.as_ref())),
                };

                HeaderRuleRecord::Remove(RemoveHeaderRuleRecord { name_id })
            }
            HeaderRule::RenameDuplicate(rule) => HeaderRuleRecord::RenameDuplicate(RenameDuplicateHeaderRuleRecord {
                name_id: interners.strings.get_or_new(rule.name.as_ref()),
                default_id: rule
                    .default
                    .as_ref()
                    .map(|default| interners.strings.get_or_new(default.as_ref())),
                rename_id: interners.strings.get_or_new(rule.rename.as_ref()),
            }),
        }
    }));
    (start..header_rules.len()).into()
}
