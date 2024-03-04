use super::super::{directive::Directive, visitor::Visitor};
use crate::{directive_de::parse_directive, rules::visitor::VisitorContext};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FederationDirective {
    pub version: FederationVersion,
}

#[derive(Debug, serde::Deserialize)]
pub enum FederationVersion {
    #[serde(rename = "2.3")]
    V2_3,
}

const FEDERATION_DIRECTIVE_NAME: &str = "federation";

impl Directive for FederationDirective {
    fn definition() -> String {
        r#"
        directive @federation(
          "The version of federation to enable."
          version: String!,
        ) on SCHEMA
        "#
        .to_string()
    }
}

pub struct FederationDirectiveVisitor;

impl<'a> Visitor<'a> for FederationDirectiveVisitor {
    fn enter_schema(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        doc: &'a engine::Positioned<engine_parser::types::SchemaDefinition>,
    ) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == FEDERATION_DIRECTIVE_NAME);

        for directive in directives {
            match parse_directive::<FederationDirective>(&directive.node, ctx.variables) {
                Ok(parsed_directive) => ctx.federation = Some(parsed_directive.version),
                Err(err) => ctx.report_error(vec![directive.pos], err.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::tests::assert_validation_error;

    #[test]
    fn test_federation_with_a_resolver() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            extend type Query {
                firstName: String! @resolver(name: "resolver")
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        insta::assert_snapshot!(registry.export_sdl(true));
    }

    #[test]
    fn test_missing_field() {
        assert_validation_error!(
            r#"extend schema @federation(version: "1")"#,
            "[1:36] unknown variant `1`, expected `2.3`"
        );
    }
}
