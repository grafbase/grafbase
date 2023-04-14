use url::Url;

use crate::directive_de::parse_directive;

use super::{directive::Directive, visitor::Visitor};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenApiDirective {
    pub name: String,
    pub url: Url,
    #[serde(rename = "schema")]
    pub schema_url: String,
    #[serde(default)]
    headers: Vec<Header>,
    #[serde(default)]
    pub transforms: OpenApiTransforms,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenApiTransforms {
    #[serde(default)]
    pub query_naming: OpenApiQueryNamingStrategy,
}

#[derive(Clone, Copy, Debug, Default, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OpenApiQueryNamingStrategy {
    OperationId,
    #[default]
    SchemaName,
}

impl OpenApiDirective {
    pub fn headers(&self) -> Vec<(String, String)> {
        self.headers
            .iter()
            .map(|header| (header.name.clone(), header.value.clone()))
            .collect()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Header {
    name: String,
    value: String,
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
          transforms: OpenApiTransforms
        ) on SCHEMA

        input OpenApiHeader {
            name: String!
            value: String!
        }

        input OpenApiTransforms {
          "How we determine the field names of the generated query type"
          queryNaming: QueryNamingStrategy = SCHEMA_NAME
        }

        enum QueryNamingStrategy {
            "We take query names directly from their OpenAPI operationId"
            OPERATION_ID
            "We take query names from the schemas they contain where possible, falling back to operationId where not"
            SCHEMA_NAME
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
            match parse_directive::<OpenApiDirective>(&directive.node, ctx.variables) {
                Ok(parsed_directive) => {
                    ctx.openapi_directives.push((parsed_directive, directive.pos));
                }
                Err(err) => ctx.report_error(vec![directive.pos], err.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{connector_parsers::MockConnectorParsers, rules::visitor::RuleError};

    use super::OpenApiQueryNamingStrategy;

    #[test]
    fn test_parsing_openapi_directive() {
        let variables = maplit::hashmap! {
                "STRIPE_API_KEY".to_string() => "i_am_a_key".to_string()
        };
        let connector_parsers = MockConnectorParsers::default();
        let schema = r#"
            extend schema
              @openapi(
                name: "stripe",
                url: "https://api.stripe.com",
                schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json",
                headers: [{ name: "authorization", value: "Bearer {{env.STRIPE_API_KEY}}"}],
              )
            "#;
        futures::executor::block_on(crate::parse(schema, &variables, &connector_parsers)).unwrap();

        insta::assert_debug_snapshot!(connector_parsers.openapi_directives.lock().unwrap(), @r###"
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
                        value: "Bearer i_am_a_key",
                    },
                ],
                transforms: OpenApiTransforms {
                    query_naming: SchemaName,
                },
            },
        ]
        "###);
    }

    #[rstest]
    #[case("OPERATION_ID", OpenApiQueryNamingStrategy::OperationId)]
    #[case("SCHEMA_NAME", OpenApiQueryNamingStrategy::SchemaName)]
    fn test_parse_naming_strategy(#[case] input: &str, #[case] expected: OpenApiQueryNamingStrategy) {
        let variables = maplit::hashmap! {
                "STRIPE_API_KEY".to_string() => "i_am_a_key".to_string()
        };
        let schema = format!(
            r#"
                    extend schema
                      @openapi(
                        name: "stripe",
                        url: "https://api.stripe.com",
                        schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json",
                        transforms: {{
                            queryNaming: {input}
                        }}
                      )
            "#
        );
        let connector_parsers = MockConnectorParsers::default();
        futures::executor::block_on(crate::parse(&schema, &variables, &connector_parsers)).unwrap();

        assert_eq!(
            connector_parsers
                .openapi_directives
                .lock()
                .unwrap()
                .first()
                .unwrap()
                .transforms
                .query_naming,
            expected
        );
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
                headers: [{ name: "authorization", value: "BLAH" }],
              )
            "#,
            "missing field `schema`"
        );
    }

    #[test]
    fn test_invalid_query_strategy() {
        assert_validation_error!(
            r#"
            extend schema
              @openapi(
                name: "stripe",
                schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json",
                url: "https://api.stripe.com",
                transforms: {queryNaming: PIES}
              )
            "#,
            "[7:29] unknown variant `PIES`, expected `OPERATION_ID` or `SCHEMA_NAME`"
        );
    }
}
