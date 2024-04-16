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
    #[serde(default)]
    allow_credentials: bool,
    allowed_headers: Option<AnyOrStringArray>,
    allowed_methods: Option<AnyOrStringArray>,
    allowed_origins: Option<AnyOrStringArray>,
    exposed_headers: Option<AnyOrStringArray>,
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
                        allow_credentials: parsed_directive.allow_credentials,
                        allowed_headers: parsed_directive.allowed_headers.map(Into::into),
                        allowed_methods: parsed_directive.allowed_methods.map(Into::into),
                        allowed_origins,
                        exposed_headers: parsed_directive.exposed_headers.map(Into::into),
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
              allowCredentials: true,
              maxAge: 88400,
              allowedHeaders: ["Authorization"],
              allowedMethods: ["GET", "POST"],
              exposedHeaders: ["Content-Type"],
              allowedOrigins: ["https://example.com/"]
            )
        "#};

        let registry = crate::parse_registry(config).unwrap();

        let expected = CorsConfig {
            max_age: Some(88400),
            allow_credentials: true,
            allowed_headers: Some(vec!["Authorization".into()]),
            allowed_methods: Some(vec!["GET".into(), "POST".into()]),
            exposed_headers: Some(vec!["Content-Type".into()]),
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
            allow_credentials: false,
            allowed_headers: None,
            allowed_methods: None,
            exposed_headers: None,
            allowed_origins: Some(vec!["*".into()]),
        };

        assert_eq!(Some(expected), registry.cors_config);
    }

    #[test]
    fn with_any_allowed_header() {
        let config = indoc! {r#"
          extend schema
            @cors(
              allowedHeaders: "*"
            )
        "#};

        let registry = crate::parse_registry(config).unwrap();

        let expected = CorsConfig {
            max_age: None,
            allow_credentials: false,
            allowed_headers: Some(vec!["*".into()]),
            allowed_methods: None,
            exposed_headers: None,
            allowed_origins: None,
        };

        assert_eq!(Some(expected), registry.cors_config);
    }

    #[test]
    fn with_any_allowed_method() {
        let config = indoc! {r#"
          extend schema
            @cors(
              allowedMethods: "*"
            )
        "#};

        let registry = crate::parse_registry(config).unwrap();

        let expected = CorsConfig {
            max_age: None,
            allow_credentials: false,
            allowed_headers: None,
            allowed_methods: Some(vec!["*".into()]),
            exposed_headers: None,
            allowed_origins: None,
        };

        assert_eq!(Some(expected), registry.cors_config);
    }

    #[test]
    fn with_any_exposed_header() {
        let config = indoc! {r#"
          extend schema
            @cors(
              exposedHeaders: "*"
            )
        "#};

        let registry = crate::parse_registry(config).unwrap();

        let expected = CorsConfig {
            max_age: None,
            allow_credentials: false,
            allowed_headers: None,
            allowed_methods: None,
            exposed_headers: Some(vec!["*".into()]),
            allowed_origins: None,
        };

        assert_eq!(Some(expected), registry.cors_config);
    }
}
