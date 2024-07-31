use engine_parser::types::SchemaDefinition;

use crate::{directive_de::parse_directive, federation::header::SubgraphHeaderRule};

use super::{
    connector_headers::Header,
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};

/// Am `@allSubgraphs` directive that can be used to pass additional
/// configuration for all subgraphs into a federated graph
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AllSubgraphsDirective {
    /// Any additional headers we want to send to all subgraphs
    #[serde(default)]
    headers: Vec<Header>,
}

impl Directive for AllSubgraphsDirective {
    fn definition() -> String {
        r#"
        directive @allSubgraphs(
          "Any additional headers we want to send to all the subgraphs"
          headers: [SubgraphHeader!]
        ) on SCHEMA
        "#
        .to_string()
    }
}

pub struct AllSubgraphsDirectiveVisitor;

impl Visitor<'_> for AllSubgraphsDirectiveVisitor {
    fn enter_schema(
        &mut self,
        ctx: &mut VisitorContext<'_>,
        doc: &engine::Positioned<SchemaDefinition>,
    ) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|directive| directive.node.name.node == "allSubgraphs")
            .collect::<Vec<_>>();

        if !ctx.registry.borrow().is_federated {
            if !directives.is_empty() {
                ctx.report_error(
                    directives
                        .into_iter()
                        .map(|directive| directive.pos)
                        .collect(),
                    "The @allSubgraphs directive is only valid in federated graphs",
                );
            }
            return;
        }

        for directive in directives {
            let directive = match parse_directive::<AllSubgraphsDirective>(directive, ctx.variables)
            {
                Ok(directive) => directive,
                Err(error) => {
                    ctx.append_errors(vec![error]);
                    return;
                }
            };

            ctx.federated_graph_config.header_rules.extend(
                directive
                    .headers
                    .into_iter()
                    .map(|header| (header.name, header.value))
                    .map(SubgraphHeaderRule::from),
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
                @allSubgraphs(
                    headers: [{name: "Auth", forward: "Authorization"}]
                )
                @allSubgraphs(
                    headers: [{name: "Auth", value: "Foo"}]
                )
                @subgraph(
                    name: "Products",
                    headers: [{name: "Other", value: "Bar"}]
                )
                @graph(type: federated)
            extend schema
                @cache(rules: [
                    {
                        maxAge: 10,
                        types: [
                            {
                                name: "TypeName",
                                fields: []
                            }
                        ]
                    }
                ])
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
                },
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
                            name: "Auth",
                            value: "Foo",
                        },
                    ),
                ],
                operation_limits: OperationLimits {
                    depth: None,
                    height: None,
                    aliases: None,
                    root_fields: None,
                    complexity: None,
                },
                global_cache_rules: GlobalCacheRules(
                    {
                        Type(
                            "TypeName",
                        ): CacheControl {
                            public: false,
                            max_age: 10,
                            stale_while_revalidate: 0,
                            invalidation_policy: None,
                            access_scopes: None,
                        },
                    },
                ),
                auth: None,
                disable_introspection: false,
                rate_limit: None,
                timeout: None,
                entity_caching: Disabled,
            },
        )
        "###);
    }

    #[test]
    fn test_errors_if_not_federated_graph() {
        assert_validation_error!(
            r#"
            extend schema
              @allSubgraphs(
                headers: [{name: "Hello", forward: true}]
              )
            "#,
            "The @allSubgraphs directive is only valid in federated graphs"
        );
    }
}
