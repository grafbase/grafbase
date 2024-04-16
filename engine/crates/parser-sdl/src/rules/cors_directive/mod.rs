use engine::registry::CorsConfig;
use engine_parser::{types::SchemaDefinition, Positioned};

use crate::directive_de::parse_directive;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};

const CORS_DIRECTIVE_NAME: &str = "cors";

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorsDirective {
    max_age: Option<u32>,
    allowed_origins: Option<AnyOrStringArray>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(expecting = "expecting string \"any\", or an array of strings")]
enum AnyOrStringArray {
    #[serde(rename = "*")]
    Any,
    #[serde(untagged)]
    Explicit(Vec<String>),
}

impl From<AnyOrStringArray> for Vec<String> {
    fn from(value: AnyOrStringArray) -> Self {
        match value {
            AnyOrStringArray::Any => vec![String::from("*")],
            AnyOrStringArray::Explicit(strings) => strings,
        }
    }
}

impl Directive for CorsDirective {
    fn definition() -> String {
        "directive @cors on SCHEMA".to_string()
    }
}

pub struct CorsVisitor;

impl<'a> Visitor<'a> for CorsVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a Positioned<SchemaDefinition>) {
        for directive in &doc.node.directives {
            if directive.node.name.node.as_str() != CORS_DIRECTIVE_NAME {
                continue;
            }

            match parse_directive::<CorsDirective>(&directive.node, ctx.variables) {
                Ok(parsed_directive) => {
                    let allowed_origins = parsed_directive
                        .allowed_origins
                        .map(Into::<Vec<_>>::into)
                        .map(|origins| {
                            origins
                                .into_iter()
                                .map(|origin| origin.strip_suffix('/').map(ToString::to_string).unwrap_or(origin))
                                .collect()
                        });

                    ctx.registry.get_mut().cors_config = Some(CorsConfig {
                        max_age: parsed_directive.max_age,
                        allowed_origins,
                    });
                }
                Err(err) => ctx.report_error(vec![directive.pos], err.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use engine::registry::CorsConfig;
    use indoc::indoc;

    #[test]
    fn with_all_settings() {
        let config = indoc! {r#"
          extend schema
            @cors(
              maxAge: 88400,
              allowedOrigins: ["https://example.com/"]
            )
        "#};

        let registry = crate::parse_registry(config).unwrap();

        let expected = CorsConfig {
            max_age: Some(88400),
            allowed_origins: Some(vec!["https://example.com".into()]),
        };

        assert_eq!(Some(expected), registry.cors_config);
    }

    #[test]
    fn with_any_allowed_origin() {
        let config = indoc! {r#"
          extend schema
            @cors(
              allowedOrigins: "*"
            )
        "#};

        let registry = crate::parse_registry(config).unwrap();

        let expected = CorsConfig {
            max_age: None,
            allowed_origins: Some(vec!["*".into()]),
        };

        assert_eq!(Some(expected), registry.cors_config);
    }
}
