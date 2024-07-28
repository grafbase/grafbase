use std::time::Duration;

use duration_str::deserialize_duration;
use engine::Positioned;
use engine_parser::types::SchemaDefinition;

use crate::{
    directive_de::parse_directive,
    rules::{
        directive::Directive,
        visitor::{Visitor, VisitorContext},
    },
};
const AUTH_V2_DIRECTIVE_NAME: &str = "authz";

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthV2Directive {
    pub providers: Vec<AuthV2Provider>,
}

impl Directive for AuthV2Directive {
    fn definition() -> String {
        format!(
            r#"
            directive @{AUTH_V2_DIRECTIVE_NAME} on SCHEMA
            "#
        )
    }
}

pub struct AuthV2DirectiveVisitor;

impl<'a> Visitor<'a> for AuthV2DirectiveVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a Positioned<SchemaDefinition>) {
        let directives = doc
            .node
            .directives
            .iter()
            .filter(|d| d.node.name.node == AUTH_V2_DIRECTIVE_NAME);

        for directive in directives {
            match parse_directive::<AuthV2Directive>(&directive.node, ctx.variables) {
                Ok(parsed_directive) => {
                    for provider in &parsed_directive.providers {
                        if provider
                            .poll_interval()
                            .filter(|duration| duration < &default_poll_interval())
                            .is_some()
                        {
                            ctx.report_error(
                                vec![directive.pos],
                                format!(
                                    "pollInterval must be at least {} seconds.",
                                    default_poll_interval().as_secs()
                                ),
                            );
                        }
                    }
                    ctx.federated_graph_config.auth = Some(parsed_directive);
                }
                Err(err) => ctx.report_error(vec![directive.pos], err.to_string()),
            }
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum AuthV2Provider {
    #[serde(rename = "jwt")]
    JWT {
        /// Used for log/error messages
        name: Option<String>,
        jwks: Jwks,
        #[serde(default)]
        header: JwtTokenHeader,
    },
    Anonymous,
}

///
/// JWT
///

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Jwks {
    pub url: url::Url,
    pub issuer: Option<String>,
    pub audience: Option<String>,
    // Using duration_str to be compatible with Apollo.
    #[serde(default = "default_poll_interval", deserialize_with = "deserialize_duration")]
    pub poll_interval: Duration,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtTokenHeader {
    #[serde(default = "default_header_name")]
    pub name: String,
    #[serde(default = "default_header_value_prefix")]
    pub value_prefix: String,
}

impl Default for JwtTokenHeader {
    fn default() -> Self {
        Self {
            name: default_header_name(),
            value_prefix: default_header_value_prefix(),
        }
    }
}

impl AuthV2Provider {
    pub fn poll_interval(&self) -> Option<Duration> {
        match self {
            AuthV2Provider::JWT { jwks, .. } => Some(jwks.poll_interval),
            AuthV2Provider::Anonymous => None,
        }
    }
}

fn default_poll_interval() -> Duration {
    Duration::from_secs(60)
}

fn default_header_name() -> String {
    "Authorization".into()
}

fn default_header_value_prefix() -> String {
    "Bearer ".into()
}

impl From<gateway_config::JwksConfig> for Jwks {
    fn from(value: gateway_config::JwksConfig) -> Self {
        Self {
            url: value.url,
            issuer: value.issuer,
            audience: value.audience,
            poll_interval: value.poll_interval,
        }
    }
}

impl From<gateway_config::AuthenticationProvider> for AuthV2Provider {
    fn from(value: gateway_config::AuthenticationProvider) -> Self {
        match value {
            gateway_config::AuthenticationProvider::Jwt(jwt) => Self::JWT {
                name: jwt.name,
                jwks: Jwks::from(jwt.jwks),
                header: JwtTokenHeader::from(jwt.header),
            },
        }
    }
}

impl From<gateway_config::AuthenticationHeader> for JwtTokenHeader {
    fn from(value: gateway_config::AuthenticationHeader) -> Self {
        Self {
            name: value.name.to_string(),
            value_prefix: value.value_prefix.to_string(),
        }
    }
}

impl From<gateway_config::AuthenticationConfig> for AuthV2Directive {
    fn from(value: gateway_config::AuthenticationConfig) -> Self {
        let providers = value.providers.into_iter().map(AuthV2Provider::from).collect();
        Self { providers }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn jwt_provider() {
        let schema = r#"
            extend schema
                @graph(type: federated)
                @authz(providers: [
                    {
                        type: "jwt",
                        jwks: { url: "https://jwks" }
                    }
                ])

        "#;

        let config = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .federated_graph_config
            .and_then(|cfg| cfg.auth);

        insta::assert_debug_snapshot!(config, @r###"
        Some(
            AuthV2Directive {
                providers: [
                    JWT {
                        name: None,
                        jwks: Jwks {
                            url: Url {
                                scheme: "https",
                                cannot_be_a_base: false,
                                username: "",
                                password: None,
                                host: Some(
                                    Domain(
                                        "jwks",
                                    ),
                                ),
                                port: None,
                                path: "/",
                                query: None,
                                fragment: None,
                            },
                            issuer: None,
                            audience: None,
                            poll_interval: 60s,
                        },
                        header: JwtTokenHeader {
                            name: "Authorization",
                            value_prefix: "Bearer ",
                        },
                    },
                ],
            },
        )
        "###);
    }

    #[test]
    fn jwt_full_provider() {
        let schema = r#"
            extend schema
                @graph(type: federated)
                @authz(providers: [
                    {
                        name: "my-jwt",
                        type: "jwt",
                        jwks: { url: "https://jwks", issuer: "auth0", audience: "grafbase" },
                        header: { name: "X-My-JWT", valuePrefix: "Bearer2 " }
                    }
                ])

        "#;

        let config = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .federated_graph_config
            .and_then(|cfg| cfg.auth);

        insta::assert_debug_snapshot!(config, @r###"
        Some(
            AuthV2Directive {
                providers: [
                    JWT {
                        name: Some(
                            "my-jwt",
                        ),
                        jwks: Jwks {
                            url: Url {
                                scheme: "https",
                                cannot_be_a_base: false,
                                username: "",
                                password: None,
                                host: Some(
                                    Domain(
                                        "jwks",
                                    ),
                                ),
                                port: None,
                                path: "/",
                                query: None,
                                fragment: None,
                            },
                            issuer: Some(
                                "auth0",
                            ),
                            audience: Some(
                                "grafbase",
                            ),
                            poll_interval: 60s,
                        },
                        header: JwtTokenHeader {
                            name: "X-My-JWT",
                            value_prefix: "Bearer2 ",
                        },
                    },
                ],
            },
        )
        "###);
    }

    #[test]
    fn multiple_provider() {
        let schema = r#"
            extend schema
                @graph(type: federated)
                @authz(providers: [
                    {
                        type: "jwt",
                        jwks: { url: "https://jwks" }
                    },
                    {
                        type: "jwt",
                        jwks: { url: "https://jwks2" }
                    }
                ])

        "#;

        let config = crate::to_parse_result_with_variables(schema, &HashMap::new())
            .unwrap()
            .federated_graph_config
            .and_then(|cfg| cfg.auth);

        insta::assert_debug_snapshot!(config, @r###"
        Some(
            AuthV2Directive {
                providers: [
                    JWT {
                        name: None,
                        jwks: Jwks {
                            url: Url {
                                scheme: "https",
                                cannot_be_a_base: false,
                                username: "",
                                password: None,
                                host: Some(
                                    Domain(
                                        "jwks",
                                    ),
                                ),
                                port: None,
                                path: "/",
                                query: None,
                                fragment: None,
                            },
                            issuer: None,
                            audience: None,
                            poll_interval: 60s,
                        },
                        header: JwtTokenHeader {
                            name: "Authorization",
                            value_prefix: "Bearer ",
                        },
                    },
                    JWT {
                        name: None,
                        jwks: Jwks {
                            url: Url {
                                scheme: "https",
                                cannot_be_a_base: false,
                                username: "",
                                password: None,
                                host: Some(
                                    Domain(
                                        "jwks2",
                                    ),
                                ),
                                port: None,
                                path: "/",
                                query: None,
                                fragment: None,
                            },
                            issuer: None,
                            audience: None,
                            poll_interval: 60s,
                        },
                        header: JwtTokenHeader {
                            name: "Authorization",
                            value_prefix: "Bearer ",
                        },
                    },
                ],
            },
        )
        "###);
    }
}
