use engine::registry::{ConnectorHeaderValue, ConnectorHeaders};
use engine_parser::types::SchemaDefinition;
use url::Url;

use super::{
    connector_headers::{Header, IntrospectionHeader},
    connector_transforms::Transforms,
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};
use crate::{directive_de::parse_directive, validations::validate_connector_name};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlDirective {
    /// A unique name for the given directive.
    ///
    /// Must be unique between all connectors.
    pub name: String,

    /// If true the GraphQL schema will be namespaced inside a dedicated object.
    #[serde(default = "default_to_true")]
    pub namespace: bool,

    pub url: Url,
    #[serde(default)]
    headers: Vec<Header>,
    #[serde(default)]
    introspection_headers: Option<Vec<IntrospectionHeader>>,

    #[serde(default)]
    pub transforms: Option<Transforms>,
}

impl GraphqlDirective {
    pub fn headers(&self) -> ConnectorHeaders {
        ConnectorHeaders::new(
            self.headers
                .iter()
                .map(|header| (header.name.clone(), header.value.clone())),
        )
    }

    pub fn introspection_headers(&self) -> Vec<(&str, &str)> {
        match &self.introspection_headers {
            Some(introspection_headers) => introspection_headers
                .iter()
                .map(|header| (header.name.as_str(), header.value.as_str()))
                .collect(),

            None => self
                .headers
                .iter()
                .filter_map(|header| match &header.value {
                    ConnectorHeaderValue::Static(value) => Some((header.name.as_str(), value.as_str())),
                    _ => None,
                })
                .collect(),
        }
    }
}

const GRAPHQL_DIRECTIVE_NAME: &str = "graphql";

impl Directive for GraphqlDirective {
    fn definition() -> String {
        r#"
        directive @graphql(
          """
          The name of the upstream GraphQL source.
          """
          name: String!

          """
          The URL of the GraphQL source.
          """
          url: Url!

          """
          Optional headers to embed in every HTTP request.
          """
          headers: [GraphqlHeader!]

          """
          Optional headers to embed in an introspection HTTP request.
          """
          introspectionHeaders: [GraphqlIntrospectionHeader!]
        ) on SCHEMA

        input GraphqlIntrospectionHeader {
            name: String!
            value: String!
        }

        input GraphqlHeader {
            name: String!
            value: String
            forward: String
        }
        "#
        .to_string()
    }
}

pub struct GraphqlVisitor;

impl<'a> Visitor<'a> for GraphqlVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a engine::Positioned<SchemaDefinition>) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == GRAPHQL_DIRECTIVE_NAME);

        for directive in directives {
            let result = parse_directive::<GraphqlDirective>(&directive.node, ctx.variables)
                .map_err(|error| error.to_string())
                .and_then(|directive| directive.validate());

            match result {
                Ok(parsed_directive) => {
                    ctx.graphql_directives.push((parsed_directive, directive.pos));
                }
                Err(err) => ctx.report_error(vec![directive.pos], err),
            }
        }
    }
}

impl GraphqlDirective {
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
    use std::collections::HashMap;

    use futures::executor::block_on;

    use crate::{connector_parsers::MockConnectorParsers, tests::assert_validation_error};

    #[test]
    fn parsing_graphql_directive() {
        let variables = HashMap::from([
            ("MY_API_KEY".to_owned(), "i_am_a_key".to_owned()),
            ("ADMIN_USER_ID".to_owned(), "root".to_owned()),
        ]);

        let connector_parsers = MockConnectorParsers::default();

        let schema = r#"
            extend schema
              @graphql(
                name: "countries",
                namespace: false,
                url: "https://countries.trevorblades.com",
                headers: [{ name: "authorization", value: "Bearer {{ env.MY_API_KEY }}"}],
                introspectionHeaders: [{ name: "x-user-id", value: "{{ env.ADMIN_USER_ID }}"}]
              )
            "#;

        block_on(crate::parse(schema, &variables, &connector_parsers)).unwrap();

        insta::assert_debug_snapshot!(connector_parsers.graphql_directives.lock().unwrap(), @r###"
        [
            GraphqlDirective {
                name: "countries",
                namespace: false,
                url: Url {
                    scheme: "https",
                    cannot_be_a_base: false,
                    username: "",
                    password: None,
                    host: Some(
                        Domain(
                            "countries.trevorblades.com",
                        ),
                    ),
                    port: None,
                    path: "/",
                    query: None,
                    fragment: None,
                },
                headers: [
                    Header {
                        name: "authorization",
                        value: Static(
                            "Bearer i_am_a_key",
                        ),
                    },
                ],
                introspection_headers: Some(
                    [
                        IntrospectionHeader {
                            name: "x-user-id",
                            value: "root",
                        },
                    ],
                ),
                transforms: None,
            },
        ]
        "###);
    }

    #[test]
    fn parsing_unnamed_graphql_directive() {
        let variables = HashMap::from([
            ("MY_API_KEY".to_owned(), "i_am_a_key".to_owned()),
            ("ADMIN_USER_ID".to_owned(), "root".to_owned()),
        ]);

        let connector_parsers = MockConnectorParsers::default();

        let schema = r#"
            extend schema
              @graphql(
                name: "Test",
                namespace: false,
                url: "https://countries.trevorblades.com",
                headers: [{ name: "authorization", value: "Bearer {{ env.MY_API_KEY }}"}],
                introspectionHeaders: [{ name: "x-user-id", value: "{{ env.ADMIN_USER_ID }}"}]
              )
            "#;

        block_on(crate::parse(schema, &variables, &connector_parsers)).unwrap();

        insta::assert_debug_snapshot!(connector_parsers.graphql_directives.lock().unwrap(), @r###"
        [
            GraphqlDirective {
                name: "Test",
                namespace: false,
                url: Url {
                    scheme: "https",
                    cannot_be_a_base: false,
                    username: "",
                    password: None,
                    host: Some(
                        Domain(
                            "countries.trevorblades.com",
                        ),
                    ),
                    port: None,
                    path: "/",
                    query: None,
                    fragment: None,
                },
                headers: [
                    Header {
                        name: "authorization",
                        value: Static(
                            "Bearer i_am_a_key",
                        ),
                    },
                ],
                introspection_headers: Some(
                    [
                        IntrospectionHeader {
                            name: "x-user-id",
                            value: "root",
                        },
                    ],
                ),
                transforms: None,
            },
        ]
        "###);
    }

    #[test]
    fn missing_field() {
        assert_validation_error!(
            r#"
            extend schema
              @graphql(
                name: "countries",
                namespace: false,
                headers: [{ name: "authorization", value: "..."}],
              )
            "#,
            "missing field `url`"
        );
    }

    #[test]
    fn invalid_header_name_type() {
        assert_validation_error!(
            r#"
            extend schema
              @graphql(
                name: "countries",
                namespace: false,
                url: "https://countries.trevorblades.com",
                headers: [{ name: 12, value: "..."}],
              )
            "#,
            "[7:26] invalid type: integer `12`, expected a string"
        );
    }

    #[test]
    fn empty_name() {
        assert_validation_error!(
            r#"
            extend schema
              @graphql(
                name: "",
                namespace: false,
                url: "https://countries.trevorblades.com",
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
              @graphql(
                name: "1234",
                namespace: false,
                url: "https://countries.trevorblades.com",
              )
            "#,
            "Connector names must be alphanumeric and cannot start with a number"
        );
    }

    #[test]
    fn invalid_introspection_header_name_type() {
        assert_validation_error!(
            r#"
            extend schema
              @graphql(
                name: "countries",
                namespace: false,
                url: "https://countries.trevorblades.com",
                introspectionHeaders: [{ name: 12, value: "..."}],
              )
            "#,
            "[7:39] invalid type: integer `12`, expected a string"
        );
    }

    #[test]
    fn test_no_introspection_headers() {
        let schema = r#"
            extend schema
              @graphql(
                name: "countries",
                namespace: false,
                url: "https://countries.trevorblades.com",
                headers: [{ name: "authorization", value: "Bearer blah"}],
                introspectionHeaders: []
              )
            "#;

        let connector_parsers = MockConnectorParsers::default();

        block_on(crate::parse(schema, &HashMap::new(), &connector_parsers)).unwrap();

        assert_eq!(
            connector_parsers.graphql_directives.lock().unwrap()[0].introspection_headers(),
            vec![]
        );
    }

    #[test]
    fn test_introspection_headers_inherits_headers_by_default() {
        let schema = r#"
            extend schema
              @graphql(
                name: "countries",
                namespace: false,
                url: "https://countries.trevorblades.com",
                headers: [{ name: "authorization", value: "Bearer blah"}],
              )
            "#;

        let connector_parsers = MockConnectorParsers::default();

        block_on(crate::parse(schema, &HashMap::new(), &connector_parsers)).unwrap();

        assert_eq!(
            connector_parsers.graphql_directives.lock().unwrap()[0].introspection_headers(),
            vec![("authorization", "Bearer blah")]
        );
    }
}
