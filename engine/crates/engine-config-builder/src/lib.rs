//! Glue crate between parser-sdl & engine-v2-config

use std::collections::BTreeMap;
use std::time::Duration;

use engine_v2_config::latest::{
    AuthConfig, AuthProviderConfig, CacheConfig, CacheConfigTarget, CacheConfigs, HeaderForward, HeaderInsert,
    HeaderRemove, HeaderRule, HeaderRuleId, NameOrPattern, OperationLimits,
};
use engine_v2_config::{
    latest::{self as config},
    VersionedConfig,
};
use federated_graph::{FederatedGraph, FederatedGraphV3, FieldId, ObjectId, SubgraphId};
use parser_sdl::federation::header::SubgraphHeaderRule;
use parser_sdl::federation::FederatedGraphConfig;
use parser_sdl::{AuthV2Provider, GlobalCacheTarget};

mod strings;

pub fn build_config(config: &FederatedGraphConfig, graph: FederatedGraph) -> VersionedConfig {
    let graph = graph.into_latest();

    let mut context = BuildContext::default();
    let mut subgraph_configs = BTreeMap::new();

    context.insert_headers(&config.header_rules);

    for (name, config) in &config.subgraphs {
        let Some(subgraph_id) = graph.find_subgraph(name) else {
            continue;
        };

        let parser_sdl::federation::SubgraphConfig {
            websocket_url,
            header_rules,
            rate_limit,
            timeout,
            ..
        } = config;

        let headers = context.insert_headers(header_rules.iter());
        let websocket_url = websocket_url.as_ref().map(|url| context.strings.intern(url));
        let subgraph_name = context.strings.intern(name);

        subgraph_configs.insert(
            subgraph_id,
            config::SubgraphConfig {
                name: subgraph_name,
                headers,
                websocket_url,
                rate_limit: rate_limit
                    .as_ref()
                    .map(|config| engine_v2_config::latest::RateLimitConfig {
                        limit: config.limit,
                        duration: config.duration,
                    }),
                timeout: *timeout,
            },
        );
    }

    let cache_config = build_cache_config(config, &graph);

    VersionedConfig::V5(config::Config {
        graph,
        strings: context.strings.into_vec(),
        header_rules: context.header_rules,
        subgraph_configs,
        cache: cache_config,
        auth: build_auth_config(config),
        operation_limits: build_operation_limits(config),
        disable_introspection: config.disable_introspection,
        rate_limit: config
            .rate_limit
            .as_ref()
            .map(|config| engine_v2_config::latest::RateLimitConfig {
                limit: config.limit,
                duration: config.duration,
            }),
        timeout: config.timeout,
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

fn build_cache_config(config: &FederatedGraphConfig, graph: &FederatedGraphV3) -> CacheConfigs {
    let mut cache_config = BTreeMap::new();

    for (target, cache_control) in config.global_cache_rules.iter() {
        match target {
            GlobalCacheTarget::Type(name) => {
                if let Some(object_id) = graph.find_object(name) {
                    cache_config.insert(
                        CacheConfigTarget::Object(object_id),
                        CacheConfig {
                            max_age: Duration::from_secs(cache_control.max_age as u64),
                            stale_while_revalidate: Duration::from_secs(cache_control.stale_while_revalidate as u64),
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
                            stale_while_revalidate: Duration::from_secs(cache_control.stale_while_revalidate as u64),
                        },
                    );
                }
            }
        }
    }

    CacheConfigs { rules: cache_config }
}

#[derive(Default)]
struct BuildContext<'a> {
    strings: strings::Strings<'a>,
    header_rules: Vec<HeaderRule>,
}

impl<'a> BuildContext<'a> {
    pub fn insert_headers(
        &mut self,
        header_rules: impl IntoIterator<Item = &'a SubgraphHeaderRule>,
    ) -> Vec<HeaderRuleId> {
        header_rules.into_iter().map(|rule| self.insert_header(rule)).collect()
    }

    pub fn insert_header(&mut self, rule: &'a SubgraphHeaderRule) -> HeaderRuleId {
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

pub trait FederatedGraphExt {
    fn find_subgraph(&self, name: &str) -> Option<SubgraphId>;
    fn find_object(&self, name: &str) -> Option<ObjectId>;
    fn find_object_field(&self, object_name: &str, field_name: &str) -> Option<FieldId>;
}

impl FederatedGraphExt for FederatedGraphV3 {
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
