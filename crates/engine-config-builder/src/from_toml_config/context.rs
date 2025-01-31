use std::collections::BTreeMap;

use engine_config::{self as config};
use federated_graph::{FederatedGraph, SubgraphId};

#[derive(Default)]
pub(super) struct BuildContext<'a> {
    pub strings: crate::strings::Strings<'a>,
    pub paths: crate::paths::Paths<'a>,
    pub header_rules: Vec<config::HeaderRule>,
    pub rate_limit: Option<config::RateLimitConfig>,
    pub subgraph_configs: BTreeMap<SubgraphId, config::SubgraphConfig>,
}

impl<'a> BuildContext<'a> {
    pub fn insert_rate_limit(&mut self, config: &'a gateway_config::RateLimitConfig) {
        let rate_limit = config::RateLimitConfig {
            global: config.global.map(|config| config::GraphRateLimit {
                limit: config.limit,
                duration: config.duration,
            }),
            storage: match config.storage {
                gateway_config::RateLimitStorage::Memory => config::RateLimitStorage::Memory,
                gateway_config::RateLimitStorage::Redis => config::RateLimitStorage::Redis,
            },
            redis: config::RateLimitRedisConfig {
                url: self.strings.intern(config.redis.url.as_str()),
                key_prefix: self.strings.intern(&config.redis.key_prefix),
                tls: config.redis.tls.as_ref().map(|config| config::RateLimitRedisTlsConfig {
                    cert: config.cert.as_ref().map(|cert| self.paths.intern(cert)),
                    key: config.key.as_ref().map(|key| self.paths.intern(key)),
                    ca: config.ca.as_ref().map(|ca| self.paths.intern(ca)),
                }),
            },
        };

        self.rate_limit = Some(rate_limit)
    }

    pub fn insert_subgraph_configs(
        &mut self,
        graph: &FederatedGraph,
        configs: impl IntoIterator<Item = (&'a String, &'a gateway_config::SubgraphConfig)>,
    ) {
        for (name, config) in configs {
            let Some(subgraph_id) = graph.find_subgraph(name) else {
                continue;
            };

            let gateway_config::SubgraphConfig {
                url,
                websocket_url,
                headers,
                rate_limit,
                timeout,
                entity_caching,
                subscriptions_protocol,
                ..
            } = config;

            let headers = self.insert_headers(headers.iter());
            let websocket_url = websocket_url.as_ref().map(|url| self.strings.intern(url.as_str()));
            let subgraph_name = self.strings.intern(name);

            let rate_limit = rate_limit.as_ref().map(|config| config::GraphRateLimit {
                limit: config.limit,
                duration: config.duration,
            });

            let retry = config.retry.as_ref().filter(|c| c.enabled).map(
                |gateway_config::RetryConfig {
                     min_per_second,
                     ttl,
                     retry_percent,
                     retry_mutations,
                     enabled: _,
                 }| config::RetryConfig {
                    min_per_second: *min_per_second,
                    ttl: *ttl,
                    retry_percent: *retry_percent,
                    retry_mutations: *retry_mutations,
                },
            );

            self.subgraph_configs.insert(
                subgraph_id,
                config::SubgraphConfig {
                    name: subgraph_name,
                    url: url.clone(),
                    headers,
                    websocket_url,
                    subscriptions_protocol: *subscriptions_protocol,
                    rate_limit,
                    timeout: *timeout,
                    retry,
                    entity_caching: entity_caching.as_ref().map(|config| {
                        if config.enabled.unwrap_or_default() {
                            config::EntityCaching::Enabled { ttl: config.ttl }
                        } else {
                            config::EntityCaching::Disabled
                        }
                    }),
                },
            );
        }
    }

    pub fn insert_headers(
        &mut self,
        header_rules: impl IntoIterator<Item = &'a gateway_config::HeaderRule>,
    ) -> Vec<config::HeaderRuleId> {
        header_rules.into_iter().map(|rule| self.insert_header(rule)).collect()
    }

    pub fn insert_header(&mut self, rule: &'a gateway_config::HeaderRule) -> config::HeaderRuleId {
        let rule = match rule {
            gateway_config::HeaderRule::Forward(ref rule) => {
                let name = self.intern_header_name(&rule.name);

                let default = rule
                    .default
                    .as_ref()
                    .map(|default| self.strings.intern(default.as_str()));

                let rename = rule.rename.as_ref().map(|rename| self.strings.intern(rename.as_str()));

                config::HeaderRule::Forward(config::HeaderForward { name, default, rename })
            }
            gateway_config::HeaderRule::Insert(ref rule) => config::HeaderRule::Insert(config::HeaderInsert {
                name: self.strings.intern(rule.name.as_str()),
                value: self.strings.intern(rule.value.as_str()),
            }),
            gateway_config::HeaderRule::Remove(ref rule) => config::HeaderRule::Remove(config::HeaderRemove {
                name: self.intern_header_name(&rule.name),
            }),
            gateway_config::HeaderRule::RenameDuplicate(ref rule) => {
                config::HeaderRule::RenameDuplicate(config::HeaderRenameDuplicate {
                    name: self.strings.intern(rule.name.as_str()),
                    default: rule
                        .default
                        .as_ref()
                        .map(|default| self.strings.intern(default.as_str())),
                    rename: self.strings.intern(rule.rename.as_str()),
                })
            }
        };

        let id = config::HeaderRuleId(self.header_rules.len());
        self.header_rules.push(rule);

        id
    }

    pub fn intern_header_name(&mut self, name: &'a gateway_config::NameOrPattern) -> config::NameOrPattern {
        match name {
            gateway_config::NameOrPattern::Pattern(ref pattern) => config::NameOrPattern::Pattern(pattern.clone()),
            gateway_config::NameOrPattern::Name(ref name) => {
                config::NameOrPattern::Name(self.strings.intern(name.as_str()))
            }
        }
    }
}

trait FederatedGraphExt {
    fn find_subgraph(&self, name: &str) -> Option<SubgraphId>;
}

impl FederatedGraphExt for FederatedGraph {
    fn find_subgraph(&self, name: &str) -> Option<SubgraphId> {
        self.subgraphs
            .iter()
            .enumerate()
            .find(|(_, subgraph)| self[subgraph.name] == name)
            .map(|(i, _)| SubgraphId::from(i))
    }
}
