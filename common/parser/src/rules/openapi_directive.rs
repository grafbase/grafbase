use url::Url;

use crate::directive_de::parse_directive;

use super::{directive::Directive, visitor::Visitor};

#[derive(Debug, serde::Deserialize)]
pub struct OpenApiDirective {
    pub name: String,
    pub url: Url,
    #[serde(rename = "schema")]
    pub schema_url: String,
    #[serde(default)]
    pub headers: Vec<Header>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

const OPENAPI_DIRECTIVE_NAME: &str = "openapi";

impl Directive for OpenApiDirective {
    fn definition() -> String {
        r#"
        directive @openapi(
          "The name of this OpenAPI source"
          name: String!
          "The URL of the API"
          url: Url!,
          "The URL of this APIs schema"
          schema: String!
          headers: [OpenApiHeader!]
        ) on SCHEMA

        input OpenApiHeader {
            name: String!
            value: String!
        }
        "#
        .to_string()
    }
}

pub struct OpenApiVisitor;

impl<'a> Visitor<'a> for OpenApiVisitor {
    fn enter_schema(
        &mut self,
        ctx: &mut super::visitor::VisitorContext<'a>,
        doc: &'a dynaql::Positioned<dynaql_parser::types::SchemaDefinition>,
    ) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == OPENAPI_DIRECTIVE_NAME);

        for directive in directives {
            match parse_directive::<OpenApiDirective>(&directive.node) {
                Ok(directive) => {
                    ctx.openapi_directives.push(directive);
                }
                Err(err) => ctx.report_error(vec![directive.pos], err.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::rules::visitor::RuleError;

    #[test]
    fn test_parsing_openapi_directive() {
        let result = crate::to_registry_with_variables(
            r#"
            extend schema
              @openapi(
                name: "stripe",
                url: "https://api.stripe.com",
                schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json",
                headers: [{ name: "authorization", value: "{{ env.STRIPE_API_KEY }}" }],
              )
            "#,
            &HashMap::new(),
        )
        .unwrap();

        insta::assert_debug_snapshot!(result.openapi_directives, @r###"
        [
            OpenApiDirective {
                name: "stripe",
                url: Url {
                    scheme: "https",
                    cannot_be_a_base: false,
                    username: "",
                    password: None,
                    host: Some(
                        Domain(
                            "api.stripe.com",
                        ),
                    ),
                    port: None,
                    path: "/",
                    query: None,
                    fragment: None,
                },
                schema_url: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json",
                headers: [
                    Header {
                        name: "authorization",
                        value: "{{ env.STRIPE_API_KEY }}",
                    },
                ],
            },
        ]
        "###);
    }

    macro_rules! assert_validation_error {
        ($schema:literal, $expected_message:literal) => {
            assert_matches!(
                crate::to_registry($schema)
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
    fn test_missing_field() {
        assert_validation_error!(
            r#"
            extend schema
              @openapi(
                name: "stripe",
                url: "https://api.stripe.com",
                headers: [{ name: "authorization", value: "{{ env.STRIPE_API_KEY }}" }],
              )
            "#,
            "missing field `schema`"
        );
    }
}
