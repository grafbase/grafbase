//! Glue crate between parser-sdl & engine-v2-config

use std::collections::BTreeMap;
use std::time::Duration;

use config::{
    AuthConfig, AuthProviderConfig, CacheConfig, CacheConfigTarget, CacheConfigs, EntityCaching, HeaderForward,
    HeaderInsert, HeaderRemove, HeaderRenameDuplicate, HeaderRule, HeaderRuleId, NameOrPattern, OperationLimits,
    SubgraphConfig,
};
use engine_v2_config::{
    latest::{self as config},
    VersionedConfig,
};
use federated_graph::{FederatedGraph, FieldId, ObjectId, SubgraphId};
use parser_sdl::federation::header::SubgraphHeaderRule;
use parser_sdl::federation::{EntityCachingConfig, FederatedGraphConfig};
use parser_sdl::{AuthV2Provider, GlobalCacheTarget};

pub fn build_with_sdl_config(config: &FederatedGraphConfig, federated_graph: FederatedGraph) -> VersionedConfig {
    let mut context = BuildContext::default();
    let default_header_rules = context.insert_headers(&config.header_rules);

    context.insert_subgraph_configs(&federated_graph, &config.subgraphs);
    context.insert_cache_config(&federated_graph, &config.global_cache_rules);

    if let Some(ref config) = config.rate_limit {
        context.insert_rate_limit(config);
    }

    VersionedConfig::V6(config::Config {
        federated_sdl: federated_graph::render_federated_sdl(&federated_graph).unwrap(),
        strings: context.strings.into_vec(),
        paths: context.paths.into_vec(),
        header_rules: context.header_rules,
        default_header_rules,
        subgraph_configs: context.subgraph_configs,
        cache: context.cache,
        auth: build_auth_config(config),
        operation_limits: build_operation_limits(config),
        disable_introspection: config.disable_introspection,
        rate_limit: context.rate_limit,
        timeout: config.timeout,
        entity_caching: match config.entity_caching {
            EntityCachingConfig::Enabled { ttl, .. } => EntityCaching::Enabled { ttl },
            _ => EntityCaching::Disabled,
        },
        retry: config.retry.map(|config| config::RetryConfig {
            min_per_second: config.min_per_second,
            ttl: config.ttl,
            retry_percent: config.retry_percent,
            retry_mutations: config.retry_mutations,
        }),
    })
}

fn build_operation_limits(config: &FederatedGraphConfig) -> OperationLimits {
    let parsed_operation_limits = &config.operation_limits;
    OperationLimits {
        depth: parsed_operation_limits.depth,
        height: parsed_operation_limits.height,
        aliases: parsed_operation_limits.aliases,
        root_fields: parsed_operation_limits.root_fields,
        complexity: parsed_operation_limits.complexity,
    }
}

// TODO: intern these
fn build_auth_config(config: &FederatedGraphConfig) -> Option<AuthConfig> {
    config.auth.as_ref().map(|auth| {
        let providers = auth
            .providers
            .iter()
            .map(|provider| match provider {
                AuthV2Provider::JWT { name, jwks, header } => AuthProviderConfig::Jwt(config::JwtConfig {
                    name: name.clone(),
                    jwks: config::JwksConfig {
                        issuer: jwks.issuer.clone(),
                        audience: jwks.audience.clone(),
                        url: jwks.url.clone(),
                        poll_interval: jwks.poll_interval,
                    },
                    header_name: header.name.clone(),
                    header_value_prefix: header.value_prefix.clone(),
                }),
                AuthV2Provider::Anonymous => AuthProviderConfig::Anonymous,
            })
            .collect();
        AuthConfig { providers }
    })
}

#[derive(Default)]
struct BuildContext<'a> {
    strings: crate::strings::Strings<'a>,
    paths: crate::paths::Paths<'a>,
    header_rules: Vec<HeaderRule>,
    rate_limit: Option<config::RateLimitConfig>,
    subgraph_configs: BTreeMap<SubgraphId, SubgraphConfig>,
    cache: CacheConfigs,
}

impl<'a> BuildContext<'a> {
    fn insert_cache_config(&mut self, graph: &FederatedGraph, rules: &parser_sdl::GlobalCacheRules<'static>) {
        let mut cache_config = BTreeMap::new();

        for (target, cache_control) in rules.iter() {
            match target {
                GlobalCacheTarget::Type(name) => {
                    if let Some(object_id) = graph.find_object(name) {
                        cache_config.insert(
                            CacheConfigTarget::Object(object_id),
                            CacheConfig {
                                max_age: Duration::from_secs(cache_control.max_age as u64),
                                stale_while_revalidate: Duration::from_secs(
                                    cache_control.stale_while_revalidate as u64,
                                ),
                            },
                        );
                    }
                }
                GlobalCacheTarget::Field(object_name, field_name) => {
                    if let Some(field_id) = graph.find_object_field(object_name, field_name) {
                        cache_config.insert(
                            CacheConfigTarget::Field(field_id),
                            CacheConfig {
                                max_age: Duration::from_secs(cache_control.max_age as u64),
                                stale_while_revalidate: Duration::from_secs(
                                    cache_control.stale_while_revalidate as u64,
                                ),
                            },
                        );
                    }
                }
            }
        }

        self.cache = CacheConfigs { rules: cache_config }
    }

    fn insert_rate_limit(&mut self, config: &'a parser_sdl::federation::RateLimitConfig) {
        let rate_limit = config::RateLimitConfig {
            global: config.global.map(|config| config::GraphRateLimit {
                limit: config.limit,
                duration: config.duration,
            }),
            storage: match config.storage {
                parser_sdl::federation::RateLimitStorage::Memory => config::RateLimitStorage::Memory,
                parser_sdl::federation::RateLimitStorage::Redis => config::RateLimitStorage::Redis,
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

    fn insert_subgraph_configs(
        &mut self,
        graph: &FederatedGraph,
        configs: impl IntoIterator<Item = (&'a String, &'a parser_sdl::federation::SubgraphConfig)>,
    ) {
        for (name, config) in configs {
            let Some(subgraph_id) = graph.find_subgraph(name) else {
                continue;
            };

            let parser_sdl::federation::SubgraphConfig {
                websocket_url,
                header_rules,
                rate_limit,
                timeout,
                entity_caching,
                ..
            } = config;

            let headers = self.insert_headers(header_rules.iter());
            let websocket_url = websocket_url.as_ref().map(|url| self.strings.intern(url));
            let subgraph_name = self.strings.intern(name);

            let rate_limit = rate_limit.as_ref().map(|config| config::GraphRateLimit {
                limit: config.limit,
                duration: config.duration,
            });

            let retry = config.retry.as_ref().map(
                |parser_sdl::federation::RetryConfig {
                     min_per_second,
                     ttl,
                     retry_percent,
                     retry_mutations,
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
                    headers,
                    websocket_url,
                    rate_limit,
                    timeout: *timeout,
                    retry,
                    entity_caching: entity_caching.as_ref().map(|config| match config {
                        EntityCachingConfig::Disabled => EntityCaching::Disabled,
                        EntityCachingConfig::Enabled { ttl, .. } => EntityCaching::Enabled { ttl: *ttl },
                    }),
                },
            );
        }
    }

    fn insert_headers(&mut self, header_rules: impl IntoIterator<Item = &'a SubgraphHeaderRule>) -> Vec<HeaderRuleId> {
        header_rules.into_iter().map(|rule| self.insert_header(rule)).collect()
    }

    fn insert_header(&mut self, rule: &'a SubgraphHeaderRule) -> HeaderRuleId {
        let rule = match rule {
            SubgraphHeaderRule::Forward(ref rule) => {
                let name = self.intern_header_name(&rule.name);
                let default = rule.default.as_ref().map(|default| self.strings.intern(default));
                let rename = rule.rename.as_ref().map(|rename| self.strings.intern(rename));

                HeaderRule::Forward(HeaderForward { name, default, rename })
            }
            SubgraphHeaderRule::Insert(ref rule) => HeaderRule::Insert(HeaderInsert {
                name: self.strings.intern(&rule.name),
                value: self.strings.intern(&rule.value),
            }),
            SubgraphHeaderRule::Remove(ref rule) => HeaderRule::Remove(HeaderRemove {
                name: self.intern_header_name(&rule.name),
            }),
            SubgraphHeaderRule::RenameDuplicate(ref rule) => HeaderRule::RenameDuplicate(HeaderRenameDuplicate {
                name: self.strings.intern(&rule.name),
                default: rule.default.as_ref().map(|default| self.strings.intern(default)),
                rename: self.strings.intern(&rule.rename),
            }),
        };

        let id = config::HeaderRuleId(self.header_rules.len());
        self.header_rules.push(rule);

        id
    }

    fn intern_header_name(&mut self, name: &'a parser_sdl::federation::header::NameOrPattern) -> NameOrPattern {
        match name {
            parser_sdl::federation::header::NameOrPattern::Pattern(ref pattern) => {
                NameOrPattern::Pattern(pattern.clone())
            }
            parser_sdl::federation::header::NameOrPattern::Name(ref name) => {
                NameOrPattern::Name(self.strings.intern(name))
            }
        }
    }
}

trait FederatedGraphExt {
    fn find_subgraph(&self, name: &str) -> Option<SubgraphId>;
    fn find_object(&self, name: &str) -> Option<ObjectId>;
    fn find_object_field(&self, object_name: &str, field_name: &str) -> Option<FieldId>;
}

impl FederatedGraphExt for FederatedGraph {
    fn find_subgraph(&self, name: &str) -> Option<SubgraphId> {
        self.subgraphs
            .iter()
            .enumerate()
            .find(|(_, subgraph)| self[subgraph.name] == name)
            .map(|(i, _)| SubgraphId(i))
    }

    fn find_object(&self, name: &str) -> Option<ObjectId> {
        self.objects
            .iter()
            .enumerate()
            .find(|(_, object)| self[object.name] == name)
            .map(|(i, _)| ObjectId(i))
    }

    fn find_object_field(&self, object_name: &str, field_name: &str) -> Option<FieldId> {
        let object = self.find_object(object_name)?;
        let fields = self[object].fields.clone();
        let start = fields.start.0;

        self[fields]
            .iter()
            .position(|field| self[field.name] == field_name)
            .map(|pos| FieldId(start + pos))
    }
}
