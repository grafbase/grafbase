use engine::registry::ConnectorHeaders;
use url::Url;

use super::{
    connector_headers::{Header, IntrospectionHeader},
    connector_transforms::Transforms,
    directive::Directive,
    visitor::Visitor,
};
use crate::{directive_de::parse_directive, validations::validate_connector_name};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenApiDirective {
    pub name: String,
    #[serde(default = "default_to_true")]
    pub namespace: bool,
    pub url: Option<Url>,
    #[serde(rename = "schema")]
    pub schema_url: String,
    #[serde(default)]
    headers: Vec<Header>,
    #[serde(default)]
    introspection_headers: Vec<IntrospectionHeader>,
    #[serde(default)]
    pub transforms: OpenApiTransforms,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenApiTransforms {
    #[serde(default)]
    pub query_naming: OpenApiQueryNamingStrategy,
    #[serde(default, flatten)]
    pub transforms: Option<Transforms>,
}

#[derive(Clone, Copy, Debug, Default, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OpenApiQueryNamingStrategy {
    OperationId,
    #[default]
    SchemaName,
}

impl OpenApiDirective {
    pub fn headers(&self) -> ConnectorHeaders {
        ConnectorHeaders::new(
            self.headers
                .iter()
                .map(|header| (header.name.clone(), header.value.clone())),
        )
    }

    pub fn introspection_headers(&self) -> Vec<(String, String)> {
        self.introspection_headers
            .iter()
            .map(|header| (header.name.clone(), header.value.clone()))
            .collect()
    }
}

const OPENAPI_DIRECTIVE_NAME: &str = "openapi";

impl Directive for OpenApiDirective {
    fn definition() -> String {
        r#"
        directive @openapi(
          "A unique name for the connector"
          name: String!
          "If true, namespaces all queries and types with the name"
          namespace: Boolean!
          "The URL of the API"
          url: Url!,
          "The URL of this APIs schema"
          schema: String!
          headers: [OpenApiHeader!]
          introspectionHeaders: [OpenApiHeaderIntrospectionHeader!]!
          transforms: OpenApiTransforms
        ) on SCHEMA

        input OpenApiHeader {
            name: String!
            value: String
            forward: String
        }

        input OpenApiHeaderIntrospectionHeader {
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
        doc: &'a engine::Positioned<engine_parser::types::SchemaDefinition>,
    ) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == OPENAPI_DIRECTIVE_NAME);

        for directive in directives {
            let result = parse_directive::<OpenApiDirective>(&directive.node, ctx.variables)
                .map_err(|error| error.to_string())
                .and_then(|directive| directive.validate());

            match result {
                Ok(parsed_directive) => {
                    ctx.openapi_directives.push((parsed_directive, directive.pos));
                }
                Err(err) => ctx.report_error(vec![directive.pos], err),
            }
        }
    }
}

impl OpenApiDirective {
    fn validate(self) -> Result<Self, String> {
        validate_connector_name(&self.name)?;

        Ok(self)
    }
}

fn default_to_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::OpenApiQueryNamingStrategy;
    use crate::{connector_parsers::MockConnectorParsers, tests::assert_validation_error};
    use rstest::rstest;
    use std::collections::HashMap;

    #[test]
    fn test_parsing_openapi_directive() {
        let variables = HashMap::from([("STRIPE_API_KEY".to_string(), "i_am_a_key".to_string())]);
        let connector_parsers = MockConnectorParsers::default();
        let schema = r#"
            extend schema
              @openapi(
                name: "Stripe",
                namespace: true,
                url: "https://api.stripe.com",
                schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json",
                headers: [{ name: "authorization", value: "Bearer {{env.STRIPE_API_KEY}}"}],
              )
            "#;
        futures::executor::block_on(crate::parse(schema, &variables, &connector_parsers)).unwrap();

        insta::assert_debug_snapshot!(connector_parsers.openapi_directives.lock().unwrap(), @r###"
        [
            OpenApiDirective {
                name: "Stripe",
                namespace: true,
                url: Some(
                    Url {
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
                ),
                schema_url: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json",
                headers: [
                    Header {
                        name: "authorization",
                        value: Static(
                            "Bearer i_am_a_key",
                        ),
                    },
                ],
                introspection_headers: [],
                transforms: OpenApiTransforms {
                    query_naming: SchemaName,
                    transforms: None,
                },
            },
        ]
        "###);
    }

    #[rstest]
    #[case("OPERATION_ID", OpenApiQueryNamingStrategy::OperationId)]
    #[case("SCHEMA_NAME", OpenApiQueryNamingStrategy::SchemaName)]
    fn test_parse_naming_strategy(#[case] input: &str, #[case] expected: OpenApiQueryNamingStrategy) {
        let variables = HashMap::from([("STRIPE_API_KEY".to_string(), "i_am_a_key".to_string())]);
        let schema = format!(
            r#"
                    extend schema
                      @openapi(
                        name: "Stripe",
                        namespace: true,
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

    #[test]
    fn test_missing_field() {
        assert_validation_error!(
            r#"
            extend schema
              @openapi(
                name: "Stripe",
                namespace: true,
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
                name: "Stripe",
                namespace: true,
                schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json",
                url: "https://api.stripe.com",
                transforms: {queryNaming: PIES}
              )
            "#,
            "[8:29] unknown variant `PIES`, expected `OPERATION_ID` or `SCHEMA_NAME`"
        );
    }

    #[test]
    fn empty_name() {
        assert_validation_error!(
            r#"
            extend schema
              @openapi(
                name: "",
                namespace: true,
                schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json",
              )
            "#,
            "Connector names cannot be empty"
        );
    }

    #[test]
    fn invalid_name() {
        assert_validation_error!(
            r#"
            extend schema
              @openapi(
                name: "1234",
                namespace: true,
                schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json",
              )
            "#,
            "Connector names must be alphanumeric and cannot start with a number"
        );
    }

    #[test]
    fn test_parsing_directive_with_duplicate_name_with_graphql() {
        assert_validation_error!(
            r#"
            extend schema
              @openapi(
                name: "Test",
                namespace: true,
                schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json",
                url: "https://api.stripe.com",
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
              @openapi(
                name: "Test",
                namespace: true,
                schema: "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json",
                url: "https://api.stripe.com",
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
}
