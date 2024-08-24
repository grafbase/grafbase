use std::time::Duration;

use engine_parser::types::SchemaDefinition;
use url::Url;

use crate::{
    directive_de::parse_directive,
    federation::{
        EntityCachingConfig,
        {header::SubgraphHeaderRule, RetryConfig as SubgraphRetryConfig},
    },
};

use super::{
    connector_headers::Header,
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};

/// A `@subgraph` directive that can be used to pass additional
/// subgraph configuration into a federated graph
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubgraphDirective {
    /// The name of the subgraph
    name: String,

    /// The URL to use for this subgraph in development
    ///
    /// In deployed environments this is ignored
    development_url: Option<String>,

    /// The URL to use for GraphQL-WS calls.
    ///
    /// This will default to the normal URL if not present.
    websocket_url: Option<Url>,

    /// Any additional headers we want to send to this subgraph
    #[serde(default)]
    headers: Vec<Header>,

    /// Timeout for requests to that subgraph
    #[serde(default, deserialize_with = "duration_str::deserialize_option_duration")]
    timeout: Option<std::time::Duration>,

    /// Whether to enable entity caching for this subgraph or not.  Defaults to the
    /// global setting.
    #[serde(default)]
    entity_caching_enabled: Option<bool>,

    /// The ttl to use for entity caching on this subgraph
    #[serde(default, deserialize_with = "duration_str::deserialize_option_duration")]
    entity_caching_ttl: Option<std::time::Duration>,

    /// Retry configuration for that subgraph
    #[serde(default)]
    retry: Option<RetryConfig>,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetryConfig {
    /// How many retries are available per second, at a minimum.
    #[serde(default)]
    pub min_per_second: Option<u32>,
    /// Each successful request to the subgraph adds to the retry budget. This setting controls for how long the budget remembers successful requests.
    #[serde(default, deserialize_with = "duration_str::deserialize_option_duration")]
    pub ttl: Option<Duration>,
    /// The fraction of the successful requests budget that can be used for retries.
    #[serde(default)]
    pub retry_percent: Option<f32>,
    /// Whether mutations should be retried at all. False by default.
    #[serde(default)]
    pub retry_mutations: Option<bool>,
}

impl Directive for SubgraphDirective {
    fn definition() -> String {
        r#"
        directive @subgraph(
          "The name of the subgraph"
          name: String!

          """
          The URL to use for this API in development

          In deployed environments this is ignored.
          """
          developmentUrl: String!

          """
          The URL to use for GraphQL-WS calls.

          This will default to the normal URL if not present.
          """
          websocketUrl: String!

          "Any additional headers we want to send to this subgraph"
          headers: [SubgraphHeader!]

          """
          Timeout for requests to that subgraph
          """
          timeout: String

          """
          Timeout for requests to that subgraph
          """
          entityCacheTtl: String

          """
          Retry configuration for that subgraph
          """
          retry: RetryConfig
        ) on SCHEMA

        input SubgraphHeader {
            name: String!
            value: String
            forward: String
        }

        input RetryConfig {
            "How many retries are available per second, at a minimum."
            minPerSecond: Int
            "Each successful request to the subgraph adds to the retry budget. This setting controls for how long the budget remembers successful requests."
            ttl: String
            "The fraction of the successful requests budget that can be used for retries."
            retryPercent: Float
            "Whether mutations should be retried at all. False by default."
            retryMutations: Boolean
        }
        "#
        .to_string()
    }
}

pub struct SubgraphDirectiveVisitor;

impl Visitor<'_> for SubgraphDirectiveVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'_>, doc: &engine::Positioned<SchemaDefinition>) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|directive| directive.node.name.node == "subgraph")
            .collect::<Vec<_>>();

        if !ctx.registry.borrow().is_federated {
            if !directives.is_empty() {
                ctx.report_error(
                    directives.into_iter().map(|directive| directive.pos).collect(),
                    "The @subgraph directive is only valid in federated graphs",
                );
            }
            return;
        }

        for directive in directives {
            let position = directive.pos;
            let directive = match parse_directive::<SubgraphDirective>(directive, ctx.variables) {
                Ok(directive) => directive,
                Err(error) => {
                    ctx.append_errors(vec![error]);
                    return;
                }
            };

            if let Some(url) = &directive.websocket_url {
                if url.scheme() != "ws" && url.scheme() != "wss" {
                    ctx.report_error(vec![position], "Websocket URLs must have a scheme of ws or wss");
                }
            }

            let subgraph = ctx
                .federated_graph_config
                .subgraphs
                .entry(directive.name.clone())
                .or_default();

            subgraph.name = directive.name;

            if let Some(url) = directive.websocket_url {
                // We want to support multiple @subgraph directives for any given subgraph
                // so if websocket_url isn't present on this one, don't set it at all
                subgraph.websocket_url = Some(url.to_string())
            }

            if let Some(url) = directive.development_url {
                subgraph.development_url = Some(url.to_string())
            }

            subgraph.entity_caching = match (directive.entity_caching_enabled, directive.entity_caching_ttl) {
                (Some(false), _) => Some(EntityCachingConfig::Disabled),
                (Some(true), ttl) => Some(EntityCachingConfig::Enabled {
                    ttl,
                    storage: Default::default(),
                }),
                (_, Some(ttl)) => Some(EntityCachingConfig::Enabled {
                    ttl: Some(ttl),
                    storage: Default::default(),
                }),
                _ => None,
            };

            subgraph.header_rules.extend(
                directive
                    .headers
                    .into_iter()
                    .map(|header| (header.name, header.value))
                    .map(SubgraphHeaderRule::from),
            );

            subgraph.timeout = directive.timeout;

            subgraph.retry = directive.retry.map(
                |RetryConfig {
                     min_per_second,
                     ttl,
                     retry_percent,
                     retry_mutations,
                 }| SubgraphRetryConfig {
                    min_per_second,
                    ttl,
                    retry_percent,
                    retry_mutations: retry_mutations.unwrap_or_default(),
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{tests::assert_validation_error, to_parse_result_with_variables};

    #[test]
    fn test_happy_path() {
        let schema = r#"
            extend schema
                @subgraph(
                    name: "Products",
                    headers: [{name: "Auth", forward: "Authorization"}]
                )
                @subgraph(
                    name: "Reviews",
                    headers: [{name: "Auth", value: "Foo"}]
                )
                @subgraph(
                    name: "Products",
                    headers: [{name: "Other", value: "Bar"}]
                )
                @graph(type: federated)
        "#;

        let result = to_parse_result_with_variables(schema, &HashMap::new()).unwrap();

        insta::assert_debug_snapshot!(result.federated_graph_config, @r###"
        Some(
            FederatedGraphConfig {
                subgraphs: {
                    "Products": SubgraphConfig {
                        name: "Products",
                        development_url: None,
                        websocket_url: None,
                        header_rules: [
                            Forward(
                                SubgraphHeaderForward {
                                    name: Name(
                                        "Authorization",
                                    ),
                                    default: None,
                                    rename: Some(
                                        "Auth",
                                    ),
                                },
                            ),
                            Insert(
                                SubgraphHeaderInsert {
                                    name: "Other",
                                    value: "Bar",
                                },
                            ),
                        ],
                        rate_limit: None,
                        timeout: None,
                        retry: None,
                        entity_caching: None,
                    },
                    "Reviews": SubgraphConfig {
                        name: "Reviews",
                        development_url: None,
                        websocket_url: None,
                        header_rules: [
                            Insert(
                                SubgraphHeaderInsert {
                                    name: "Auth",
                                    value: "Foo",
                                },
                            ),
                        ],
                        rate_limit: None,
                        timeout: None,
                        retry: None,
                        entity_caching: None,
                    },
                },
                header_rules: [],
                operation_limits: OperationLimits {
                    depth: None,
                    height: None,
                    aliases: None,
                    root_fields: None,
                    complexity: None,
                },
                global_cache_rules: GlobalCacheRules(
                    {},
                ),
                auth: None,
                disable_introspection: false,
                rate_limit: None,
                timeout: None,
                entity_caching: Disabled,
                retry: None,
            },
        )
        "###);
    }

    #[test]
    fn test_errors_if_not_federated_graph() {
        assert_validation_error!(
            r#"
            extend schema
              @subgraph(
                name: "blah",
                headers: [{name: "Hello", forward: true}]
              )
            "#,
            "The @subgraph directive is only valid in federated graphs"
        );
    }
}
