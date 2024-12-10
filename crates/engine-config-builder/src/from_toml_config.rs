mod context;

use federated_graph::FederatedGraph;

pub fn build_with_toml_config(config: &gateway_config::Config, graph: FederatedGraph) -> engine_config::Config {
    let mut context = context::BuildContext::default();

    let default_header_rules = context.insert_headers(&config.headers);
    context.insert_subgraph_configs(&graph, &config.subgraphs);

    if let Some(ref rate_limit) = config.gateway.rate_limit {
        context.insert_rate_limit(rate_limit);
    }

    engine_config::Config {
        graph,
        strings: context.strings.into_vec(),
        paths: context.paths.into_vec(),
        header_rules: context.header_rules,
        default_header_rules,
        subgraph_configs: context.subgraph_configs,
        auth: build_auth_config(config),
        operation_limits: build_operation_limits(config),
        disable_introspection: !config.graph.introspection.unwrap_or_default(),
        rate_limit: context.rate_limit,
        timeout: config.gateway.timeout,
        entity_caching: if config.entity_caching.enabled.unwrap_or_default() {
            engine_config::EntityCaching::Enabled {
                ttl: config.entity_caching.ttl,
            }
        } else {
            engine_config::EntityCaching::Disabled
        },
        retry: config.gateway.retry.enabled.then_some(engine_config::RetryConfig {
            min_per_second: config.gateway.retry.min_per_second,
            ttl: config.gateway.retry.ttl,
            retry_percent: config.gateway.retry.retry_percent,
            retry_mutations: config.gateway.retry.retry_mutations,
        }),
        batching: engine_config::BatchingConfig {
            enabled: config.gateway.batching.enabled,
            limit: config.gateway.batching.limit.map(usize::from),
        },
        complexity_control: build_complexity_control(&config.complexity_control),
        response_extension: config
            .telemetry
            .exporters
            .response_extension
            .clone()
            .unwrap_or_default()
            .into(),
        apq: engine_config::AutomaticPersistedQueries {
            enabled: config.apq.enabled,
        },
        executable_document_limit_bytes: config
            .executable_document_limit
            .bytes()
            .try_into()
            .expect("executable document limit should not be negative"),
    }
}

fn build_auth_config(config: &gateway_config::Config) -> Option<engine_config::AuthConfig> {
    config.authentication.as_ref().map(|auth| {
        let providers = auth
            .providers
            .iter()
            .map(|provider| match provider {
                gateway_config::AuthenticationProvider::Jwt(provider) => {
                    engine_config::AuthProviderConfig::Jwt(engine_config::JwtConfig {
                        name: provider.name.clone(),
                        jwks: engine_config::JwksConfig {
                            issuer: provider.jwks.issuer.clone(),
                            audience: provider.jwks.audience.clone(),
                            url: provider.jwks.url.clone(),
                            poll_interval: provider.jwks.poll_interval,
                        },
                        header_name: provider.header.name.to_string(),
                        header_value_prefix: provider.header.value_prefix.to_string(),
                    })
                }
                gateway_config::AuthenticationProvider::Anonymous => engine_config::AuthProviderConfig::Anonymous,
            })
            .collect();

        engine_config::AuthConfig { providers }
    })
}

fn build_operation_limits(config: &gateway_config::Config) -> engine_config::OperationLimits {
    let Some(parsed_operation_limits) = &config.operation_limits else {
        return engine_config::OperationLimits::default();
    };

    engine_config::OperationLimits {
        depth: parsed_operation_limits.depth,
        height: parsed_operation_limits.height,
        aliases: parsed_operation_limits.aliases,
        root_fields: parsed_operation_limits.root_fields,
        complexity: parsed_operation_limits.complexity,
    }
}

fn build_complexity_control(config: &gateway_config::ComplexityControlConfig) -> engine_config::ComplexityControl {
    use engine_config::ComplexityControl;
    use gateway_config::ComplexityControlMode;

    let list_size = |config: &gateway_config::ComplexityControlConfig| {
        config.list_size.unwrap_or_else(|| {
            tracing::warn!("Complexity control enabled without setting list_size.  Assuming a list_size of 10");
            10
        })
    };

    match config.mode {
        None => ComplexityControl::Disabled,
        Some(ComplexityControlMode::Enforce) if config.limit.is_some() => ComplexityControl::Enforce {
            limit: config.limit.unwrap(),
            list_size: list_size(config),
        },
        Some(ComplexityControlMode::Enforce) => {
            tracing::warn!(
                "Complexity control is configured to enforce limits but a limit was not configured.  Complexity will only be measured"
            );
            ComplexityControl::Measure {
                limit: config.limit,
                list_size: list_size(config),
            }
        }
        Some(ComplexityControlMode::Measure) => ComplexityControl::Measure {
            limit: config.limit,
            list_size: list_size(config),
        },
    }
}
