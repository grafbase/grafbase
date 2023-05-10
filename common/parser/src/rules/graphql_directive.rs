use std::collections::HashMap;

use dynaql_parser::types::SchemaDefinition;
use url::Url;

use crate::directive_de::parse_directive;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlDirective {
    pub name: String,
    pub url: Url,
    #[serde(default)]
    pub headers: Vec<Header>,
}

impl GraphqlDirective {
    pub fn headers(&self) -> HashMap<String, String> {
        self.headers
            .iter()
            .map(|header| (header.name.clone(), header.value.clone()))
            .collect()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
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
          url: Url!,

          """
          Optional headers to embed in every HTTP request.
          """
          headers: [GraphqlHeader!]
        ) on SCHEMA

        input GraphqlHeader {
            name: String!
            value: String!
        }
        "#
        .to_string()
    }
}

pub struct GraphqlVisitor;

impl<'a> Visitor<'a> for GraphqlVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a dynaql::Positioned<SchemaDefinition>) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == GRAPHQL_DIRECTIVE_NAME);

        for directive in directives {
            match parse_directive::<GraphqlDirective>(&directive.node, ctx.variables) {
                Ok(parsed_directive) => {
                    ctx.graphql_directives.push((parsed_directive, directive.pos));
                }
                Err(err) => ctx.report_error(vec![directive.pos], err.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use futures::executor::block_on;
    use rstest::rstest;

    use crate::{connector_parsers::MockConnectorParsers, rules::visitor::RuleError};

    #[test]
    fn parsing_graphql_directive() {
        let variables = HashMap::from([("MY_API_KEY".to_owned(), "i_am_a_key".to_owned())]);
        let connector_parsers = MockConnectorParsers::default();
        let schema = r#"
            extend schema
              @graphql(
                name: "countries",
                url: "https://countries.trevorblades.com",
                headers: [{ name: "authorization", value: "Bearer {{env.MY_API_KEY}}"}],
              )
            "#;

        block_on(crate::parse(schema, &variables, &connector_parsers)).unwrap();

        insta::assert_debug_snapshot!(connector_parsers.graphql_directives.lock().unwrap(), @r###"
        [
            GraphqlDirective {
                name: "countries",
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
                        value: "Bearer i_am_a_key",
                    },
                ],
            },
        ]
        "###);
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
            );
        };
    }

    #[test]
    fn missing_field() {
        assert_validation_error!(
            r#"
            extend schema
              @graphql(
                url: "https://countries.trevorblades.com",
                headers: [{ name: "authorization", value: "..."}],
              )
            "#,
            "missing field `name`"
        );
    }

    #[test]
    fn invalid_header_name_type() {
        assert_validation_error!(
            r#"
            extend schema
              @graphql(
                url: "https://countries.trevorblades.com",
                headers: [{ name: 12, value: "..."}],
              )
            "#,
            "[5:26] invalid type: integer `12`, expected a string"
        );
    }
}
