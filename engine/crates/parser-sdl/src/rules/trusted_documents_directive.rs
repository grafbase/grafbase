use engine_parser::types::SchemaDefinition;
use itertools::Itertools;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};
use crate::directive_de::parse_directive;

const TRUSTED_DOCUMENTS_DIRECTIVE_NAME: &str = "trustedDocuments";

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustedDocumentsDirective {
    bypass_header_name: Option<String>,
    bypass_header_value: Option<String>,
}

impl From<TrustedDocumentsDirective> for engine::registry::TrustedDocuments {
    fn from(value: TrustedDocumentsDirective) -> Self {
        Self {
            bypass_header_name: value.bypass_header_name,
            bypass_header_value: value.bypass_header_value,
        }
    }
}

impl Directive for TrustedDocumentsDirective {
    fn definition() -> String {
        r#"
        directive @trustedDocuments(
          """
          An HTTP header that can be used to send arbitrary queries.
          """
          bypassHeaderName: String

          """
          The value that must be taken by the header specified in bypassHeaderName.
          """
          bypassHeaderValue: String
        ) on SCHEMA
        "#
        .to_string()
    }
}

pub struct TrustedDocumentsVisitor;

impl<'a> Visitor<'a> for TrustedDocumentsVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a engine::Positioned<SchemaDefinition>) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == TRUSTED_DOCUMENTS_DIRECTIVE_NAME);

        match directives.at_most_one() {
            Ok(None) => (),
            Ok(Some(directive)) => {
                match parse_directive::<TrustedDocumentsDirective>(&directive.node, ctx.variables)
                    .map_err(|error| error.to_string())
                {
                    Ok(trusted_documents) => {
                        ctx.trusted_documents_directive = Some(trusted_documents);
                    }
                    Err(error) => {
                        ctx.report_error(vec![directive.pos], error);
                    }
                }
            }
            Err(duplicates) => {
                for duplicate_directive in duplicates.skip(1) {
                    ctx.report_error(
                        vec![duplicate_directive.pos],
                        "The @trustedDocuments directive can only appear once",
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::connector_parsers::MockConnectorParsers;
    use futures::executor::block_on;
    use std::collections::HashMap;

    #[test]
    fn parsing_trusted_documents_basic() {
        let schema = r#"
            extend schema
              @trustedDocuments
            "#;

        let result = block_on(crate::parse(schema, &HashMap::new(), &MockConnectorParsers::default())).unwrap();

        insta::assert_debug_snapshot!(result.registry.trusted_documents, @r###"
        Some(
            TrustedDocuments {
                bypass_header_name: None,
                bypass_header_value: None,
            },
        )
        "###);
    }

    #[test]
    fn parsing_trusted_documents_with_bypass_header() {
        let schema = r#"
            extend schema
              @trustedDocuments(bypassHeaderName: "x-special-header", bypassHeaderValue: "special")
            "#;

        let result = block_on(crate::parse(schema, &HashMap::new(), &MockConnectorParsers::default())).unwrap();

        insta::assert_debug_snapshot!(result.registry.trusted_documents, @r###"
        Some(
            TrustedDocuments {
                bypass_header_name: Some(
                    "x-special-header",
                ),
                bypass_header_value: Some(
                    "special",
                ),
            },
        )
        "###);
    }
}
