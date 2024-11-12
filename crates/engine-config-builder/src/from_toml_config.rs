mod context;

use federated_graph::FederatedGraph;

pub fn build_with_toml_config(config: &gateway_config::Config, graph: FederatedGraph) -> engine_v2_config::Config {
    let mut context = context::BuildContext::default();

    let default_header_rules = context.insert_headers(&config.headers);
    context.insert_subgraph_configs(&graph, &config.subgraphs);

    if let Some(ref rate_limit) = config.gateway.rate_limit {
        context.insert_rate_limit(rate_limit);
    }

    engine_v2_config::Config {
        graph,
        strings: context.strings.into_vec(),
        paths: context.paths.into_vec(),
        header_rules: context.header_rules,
        default_header_rules,
        subgraph_configs: context.subgraph_configs,
        auth: build_auth_config(config),
        operation_limits: build_operation_limits(config),
        disable_introspection: !config.graph.introspection,
        rate_limit: context.rate_limit,
        timeout: config.gateway.timeout,
        entity_caching: if config.entity_caching.enabled.unwrap_or_default() {
            engine_v2_config::EntityCaching::Enabled {
                ttl: config.entity_caching.ttl,
            }
        } else {
            engine_v2_config::EntityCaching::Disabled
        },
        retry: config.gateway.retry.enabled.then_some(engine_v2_config::RetryConfig {
            min_per_second: config.gateway.retry.min_per_second,
            ttl: config.gateway.retry.ttl,
            retry_percent: config.gateway.retry.retry_percent,
            retry_mutations: config.gateway.retry.retry_mutations,
        }),
        batching: engine_v2_config::BatchingConfig {
            enabled: config.gateway.batching.enabled,
            limit: config.gateway.batching.limit.map(usize::from),
        },
    }
}

fn build_auth_config(config: &gateway_config::Config) -> Option<engine_v2_config::AuthConfig> {
    config.authentication.as_ref().map(|auth| {
        let providers = auth
            .providers
            .iter()
            .map(|provider| match provider {
                gateway_config::AuthenticationProvider::Jwt(provider) => {
                    engine_v2_config::AuthProviderConfig::Jwt(engine_v2_config::JwtConfig {
                        name: provider.name.clone(),
                        jwks: engine_v2_config::JwksConfig {
                            issuer: provider.jwks.issuer.clone(),
                            audience: provider.jwks.audience.clone(),
                            url: provider.jwks.url.clone(),
                            poll_interval: provider.jwks.poll_interval,
                        },
                        header_name: provider.header.name.to_string(),
                        header_value_prefix: provider.header.value_prefix.to_string(),
                    })
                }
                gateway_config::AuthenticationProvider::Anonymous => engine_v2_config::AuthProviderConfig::Anonymous,
            })
            .collect();

        engine_v2_config::AuthConfig { providers }
    })
}

fn build_operation_limits(config: &gateway_config::Config) -> engine_v2_config::OperationLimits {
    let Some(parsed_operation_limits) = &config.operation_limits else {
        return engine_v2_config::OperationLimits::default();
    };

    engine_v2_config::OperationLimits {
        depth: parsed_operation_limits.depth,
        height: parsed_operation_limits.height,
        aliases: parsed_operation_limits.aliases,
        root_fields: parsed_operation_limits.root_fields,
        complexity: parsed_operation_limits.complexity,
    }
}
