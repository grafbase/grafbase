use engine_parser::types::SchemaDefinition;
use url::Url;

use crate::directive_de::parse_directive;

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

    /// The URL to use for GraphQL-WS calls.
    ///
    /// This will default to the normal URL if not present.
    websocket_url: Option<Url>,

    /// Any additional headers we want to send to this subgraph
    #[serde(default)]
    headers: Vec<Header>,
}

impl Directive for SubgraphDirective {
    fn definition() -> String {
        r#"
        directive @subgraph(
          "The name of the subgraph"
          name: String!

          """
          The URL to use for GraphQL-WS calls.

          This will default to the normal URL if not present.
          """
          websocketUrl: String!

          "Any additional headers we want to send to this subgraph"
          headers: [SubgraphHeader!]
        ) on SCHEMA

        input SubgraphHeader {
            name: String!
            value: String
            forward: String
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
            subgraph.websocket_url = directive.websocket_url.map(|url| url.to_string());
            subgraph.headers.extend(
                directive
                    .headers
                    .into_iter()
                    .map(|header| (header.name, header.value.into())),
            )
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
                        websocket_url: None,
                        headers: [
                            (
                                "Auth",
                                Forward(
                                    "Authorization",
                                ),
                            ),
                            (
                                "Other",
                                Static(
                                    "Bar",
                                ),
                            ),
                        ],
                    },
                    "Reviews": SubgraphConfig {
                        name: "Reviews",
                        websocket_url: None,
                        headers: [
                            (
                                "Auth",
                                Static(
                                    "Foo",
                                ),
                            ),
                        ],
                    },
                },
                default_headers: [],
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
