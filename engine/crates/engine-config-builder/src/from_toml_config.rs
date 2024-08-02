use engine_v2_config::VersionedConfig;
use federated_graph::FederatedGraph;
use gateway_config::{Config, RetryConfig};
use parser_sdl::federation::{header::SubgraphHeaderRule, FederatedGraphConfig};

use crate::build_with_sdl_config;

pub fn build_with_toml_config(config: &Config, graph: FederatedGraph) -> VersionedConfig {
    let mut graph_config = FederatedGraphConfig::default();

    if let Some(limits_config) = config.operation_limits {
        graph_config.operation_limits = limits_config.into();
    }

    if let Some(auth_config) = config.authentication.clone() {
        graph_config.auth = Some(auth_config.into());
    }

    graph_config.timeout = config.gateway.timeout;
    graph_config.disable_introspection = !config.graph.introspection;

    graph_config.header_rules = config
        .headers
        .clone()
        .into_iter()
        .map(SubgraphHeaderRule::from)
        .collect();

    graph_config.rate_limit = config.gateway.rate_limit.clone().map(Into::into);
    graph_config.entity_caching = config.entity_caching.clone().into();

    graph_config.subgraphs = config
        .subgraphs
        .clone()
        .into_iter()
        .map(|(name, subgraph_config)| {
            let header_rules = subgraph_config
                .headers
                .into_iter()
                .map(SubgraphHeaderRule::from)
                .collect();

            let config = parser_sdl::federation::SubgraphConfig {
                name: name.clone(),
                websocket_url: subgraph_config.websocket_url.map(|url| url.to_string()),
                header_rules,
                development_url: None,
                rate_limit: subgraph_config.rate_limit.map(Into::into),
                timeout: subgraph_config.timeout.or(config.gateway.subgraph_timeout),
                entity_caching: subgraph_config.entity_caching.map(Into::into),
                retry: retry_config(subgraph_config.retry, config.gateway.retry),
            };

            (name, config)
        })
        .collect();

    build_with_sdl_config(&graph_config, graph)
}

fn retry_config(
    subgraph_retry: Option<RetryConfig>,
    global_retry: RetryConfig,
) -> Option<parser_sdl::federation::RetryConfig> {
    let retry = match subgraph_retry {
        Some(retry) if retry.enabled => Some(retry),
        None if global_retry.enabled => Some(global_retry),
        _ => None,
    };

    retry.map(|retry| parser_sdl::federation::RetryConfig {
        min_per_second: retry.min_per_second,
        ttl: retry.ttl,
        retry_percent: retry.retry_percent,
        retry_mutations: retry.retry_mutations,
    })
}

#[cfg(test)]
mod tests {
    use gateway_config::RetryConfig;

    #[test]
    fn no_retry_enabled() {
        let result = super::retry_config(None, RetryConfig::default());
        assert_eq!(None, result);
    }

    #[test]
    fn global_retry_enabled() {
        let result = super::retry_config(
            None,
            RetryConfig {
                enabled: true,
                ..Default::default()
            },
        );

        assert_eq!(Some(parser_sdl::federation::RetryConfig::default()), result);
    }

    #[test]
    fn global_retry_enabled_subgraph_retry_disabled() {
        let global = RetryConfig {
            enabled: true,
            ..Default::default()
        };

        let subgraph = RetryConfig {
            enabled: false,
            ..Default::default()
        };

        let result = super::retry_config(Some(subgraph), global);

        assert_eq!(None, result);
    }

    #[test]
    fn global_retry_enabled_subgraph_retry_enabled() {
        let global = RetryConfig {
            enabled: true,
            ..Default::default()
        };

        let subgraph = RetryConfig {
            enabled: true,
            min_per_second: Some(10),
            ..Default::default()
        };

        let result = super::retry_config(Some(subgraph), global);

        let expected = parser_sdl::federation::RetryConfig {
            min_per_second: Some(10),
            ..Default::default()
        };

        assert_eq!(Some(expected), result);
    }

    #[test]
    fn global_retry_disabled_subgraph_retry_enabled() {
        let global = RetryConfig {
            enabled: false,
            ..Default::default()
        };

        let subgraph = RetryConfig {
            enabled: true,
            min_per_second: Some(10),
            ..Default::default()
        };

        let result = super::retry_config(Some(subgraph), global);

        let expected = parser_sdl::federation::RetryConfig {
            min_per_second: Some(10),
            ..Default::default()
        };

        assert_eq!(Some(expected), result);
    }
}
