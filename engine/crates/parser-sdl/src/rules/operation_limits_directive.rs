use engine::registry::OperationLimits;
use engine_parser::types::SchemaDefinition;
use itertools::Itertools;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};
use crate::directive_de::parse_directive;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationLimitsDirective {
    depth: Option<u16>,
    height: Option<u16>,
    aliases: Option<u16>,
    root_fields: Option<u16>,
    complexity: Option<u16>,
}

impl From<OperationLimitsDirective> for OperationLimits {
    fn from(
        OperationLimitsDirective {
            depth,
            height,
            aliases,
            root_fields,
            complexity,
        }: OperationLimitsDirective,
    ) -> Self {
        OperationLimits {
            depth,
            height,
            aliases,
            root_fields,
            complexity,
        }
    }
}

const OPERATION_LIMITS_DIRECTIVE_NAME: &str = "operationLimits";

impl Directive for OperationLimitsDirective {
    fn definition() -> String {
        r#"
        directive @operationLimits(
          """
          The maximum depth limit.
          """
          depth: Int

          """
          The maximum height limit.
          """
          height: Int

          """
          The maximum aliases' number limit.
          """
          aliases: Int

          """
          The maximum root fields' limit.
          """
          rootFields: Int

          """
          The maximum total complexity limit.
          """
          complexity: Int
        ) on SCHEMA
        "#
        .to_string()
    }
}

pub struct OperationLimitsVisitor;

impl<'a> Visitor<'a> for OperationLimitsVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a engine::Positioned<SchemaDefinition>) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == OPERATION_LIMITS_DIRECTIVE_NAME);

        match directives.at_most_one() {
            Ok(operation_limits_directive) => {
                if let Some(directive) = operation_limits_directive {
                    match parse_directive::<OperationLimitsDirective>(&directive.node, ctx.variables)
                        .map_err(|error| error.to_string())
                        .and_then(|directive| directive.validate())
                    {
                        Ok(operation_limits) => {
                            ctx.operation_limits_directive = Some(operation_limits);
                        }
                        Err(error) => {
                            ctx.report_error(vec![directive.pos], error);
                        }
                    }
                }
            }
            Err(duplicates) => {
                for duplicate_directive in duplicates.skip(1) {
                    ctx.report_error(
                        vec![duplicate_directive.pos],
                        "The @operationLimits can only appear once",
                    );
                }
            }
        }
    }
}

impl OperationLimitsDirective {
    fn validate(self) -> Result<Self, String> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use futures::executor::block_on;

    use crate::connector_parsers::MockConnectorParsers;

    #[test]
    fn parsing_operation_limits_directive() {
        let schema = r#"
            extend schema
              @operationLimits(
                depth: 5,
                aliases: 10,
                complexity: 100,
              )
            "#;

        let result = block_on(crate::parse(
            schema,
            &HashMap::new(),
            false,
            &MockConnectorParsers::default(),
        ))
        .unwrap();

        insta::assert_debug_snapshot!(result.registry.operation_limits, @r###"
        OperationLimits {
            depth: Some(
                5,
            ),
            height: None,
            aliases: Some(
                10,
            ),
            root_fields: None,
            complexity: Some(
                100,
            ),
        }
        "###);
    }
}
