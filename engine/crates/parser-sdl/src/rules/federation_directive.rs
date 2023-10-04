use super::{directive::Directive, visitor::Visitor};
use crate::directive_de::parse_directive;

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
        ctx: &mut super::visitor::VisitorContext<'a>,
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

    use crate::RuleError;

    #[cfg(comebacktothisone)]
    #[test]
    fn test_federation_with_no_models() {
        let schema = r#"extend schema @federation(version: "2.3")"#;
        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;
        // TODO: This fails.  Not sure if it should?
        assert_eq!(registry.export_sdl(false), registry.export_sdl(true));
    }

    #[test]
    fn test_federation_with_a_model() {
        let schema = r#"
            extend schema @federation(version: "2.3")

            type User @model {
                id: ID!
                firstName: String!
            }
        "#;

        let registry = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .registry;

        insta::assert_display_snapshot!(registry.export_sdl(true));
    }

    macro_rules! assert_validation_error {
        ($schema:literal, $expected_message:literal) => {
            assert_matches!(
                crate::parse_registry($schema)
                    .err()
                    .and_then(crate::Error::validation_errors)
                    // We don't care whether there are more errors or not.
                    // It only matters that we find the expected error.
                    .and_then(|errors| errors.into_iter().next()),
                Some(RuleError { message, .. }) => {
                    assert_eq!(message, $expected_message);
                }
            )
        };
    }

    #[test]
    fn test_missing_field() {
        assert_validation_error!(
            r#"extend schema @federation(version: "1")"#,
            "[1:36] unknown variant `1`, expected `2.3`"
        );
    }
}
