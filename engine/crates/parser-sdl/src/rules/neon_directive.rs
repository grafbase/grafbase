use engine::Positioned;
use engine_parser::types::SchemaDefinition;

use crate::directive_de::parse_directive;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};

const NEON_DIRECTIVE_NAME: &str = "neon";

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeonDirective {
    name: String,
    url: String,
    #[serde(default = "default_to_true")]
    namespace: bool,
}

fn default_to_true() -> bool {
    true
}

impl NeonDirective {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn postgresql_url(&self) -> &str {
        &self.url
    }

    pub fn namespace(&self) -> bool {
        self.namespace
    }
}

impl Directive for NeonDirective {
    fn definition() -> String {
        r#"
        directive @neon(
          """
          A unique name for the given directive.
          """
          name: String!

          """
          The full connection string to the database.
          """
          url: String!
          
          """
          If true, namespaces queries and mutations with the
          connector name. Defaults to true.
          """
          namespace: Boolean
        ) on SCHEMA
        "#
        .to_string()
    }
}

pub struct NeonVisitor;

impl<'a> Visitor<'a> for NeonVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a Positioned<SchemaDefinition>) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == NEON_DIRECTIVE_NAME);

        for directive in directives {
            match parse_directive::<NeonDirective>(&directive.node, ctx.variables) {
                Ok(parsed_directive) => ctx.neon_directives.push((parsed_directive, directive.pos)),
                Err(err) => ctx.report_error(vec![directive.pos], err.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use futures::executor::block_on;

    use crate::{connector_parsers::MockConnectorParsers, rules::visitor::RuleError};

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
            );
        };
    }

    #[test]
    fn parsing_neon_directive() {
        let variables = HashMap::from([(
            "NEON_CONNECTION_STRING".to_string(),
            "postgres://postgres:grafbase@localhost:5432/postgres".to_string(),
        )]);

        let schema = r#"
            extend schema
              @neon(
                name: "possu",
                namespace: true,
                url: "{{ env.NEON_CONNECTION_STRING }}",
              )
            "#;

        let connector_parsers = MockConnectorParsers::default();

        block_on(crate::parse(schema, &variables, &connector_parsers)).unwrap();

        insta::assert_debug_snapshot!(connector_parsers.neon_directives.lock().unwrap(), @r###"
        [
            NeonDirective {
                name: "possu",
                url: "postgres://postgres:grafbase@localhost:5432/postgres",
                namespace: true,
            },
        ]
        "###);
    }

    #[test]
    fn test_parsing_directive_with_duplicate_name_with_graphql() {
        assert_validation_error!(
            r#"
            extend schema
              @neon(
                name: "Test",
                namespace: true,
                url: "postgres://postgres:grafbase@localhost:5432/postgres",
              )

            extend schema
              @graphql(
                name: "Test",
                namespace: true,
                url: "https://countries.trevorblades.com",
              )
            "#,
            "Name \"Test\" is not unique. A connector must have a unique name."
        );
    }

    #[test]
    fn test_parsing_directive_with_duplicate_name_with_mongo() {
        assert_validation_error!(
            r#"
            extend schema
              @neon(
                name: "Test",
                namespace: true,
                url: "postgres://postgres:grafbase@localhost:5432/postgres",
              )

            extend schema
              @mongodb(
                 name: "Test",
                 apiKey: "TEST"
                 url: "TEST"
                 dataSource: "TEST"
                 database: "TEST"
                 namespace: true,
              )
            "#,
            "Name \"Test\" is not unique. A connector must have a unique name."
        );
    }

    #[test]
    fn test_parsing_directive_with_duplicate_name_with_openapi() {
        assert_validation_error!(
            r#"
            extend schema
              @neon(
                name: "Test",
                namespace: true,
                url: "postgres://postgres:grafbase@localhost:5432/postgres",
              )

            extend schema
              @openapi(
                name: "Test",
                namespace: true,
                schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json",
                url: "https://api.stripe.com",
              )
            "#,
            "Name \"Test\" is not unique. A connector must have a unique name."
        );
    }
}
