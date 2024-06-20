use std::net::IpAddr;
use std::time::Duration;

use itertools::Itertools;
use serde::{Deserialize, Deserializer};

use engine_parser::{types::SchemaDefinition, Positioned};
use registry_v2::rate_limiting::{Header, Jwt, RateLimitConfig};

use crate::directive_de::parse_directive;

use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};

const RATE_LIMITING_DIRECTIVE_NAME: &str = "rateLimiting";

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RateLimitingDirective {
    #[serde(default)]
    pub rules: Vec<RateLimitRule>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RateLimitRule {
    pub condition: RateLimitRuleCondition,
    pub name: String,
    pub limit: u32,
    #[serde(deserialize_with = "from_number")]
    pub duration: Duration,
}

pub fn from_number<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    u64::deserialize(deserializer).map(Duration::from_secs)
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum RateLimitRuleCondition {
    Header { headers: Vec<Header> },
    GraphqlOperation { operations: Vec<String> },
    Ip { ips: Vec<IpAddr> },
    JwtClaim { jwt_claims: Vec<Jwt> },
}

impl Directive for RateLimitingDirective {
    fn definition() -> String {
        "directive @rateLimiting on SCHEMA".to_string()
    }
}

pub struct RateLimitingVisitor;

impl<'a> Visitor<'a> for RateLimitingVisitor {
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a Positioned<SchemaDefinition>) {
        for directive in &doc.node.directives {
            if directive.node.name.node.as_str() != RATE_LIMITING_DIRECTIVE_NAME {
                continue;
            }

            match parse_directive::<RateLimitingDirective>(&directive.node, ctx.variables) {
                Ok(parsed_directive) => {
                    ctx.registry.get_mut().rate_limiting = RateLimitConfig {
                        rules: parsed_directive
                            .rules
                            .into_iter()
                            .map(|rule| registry_v2::rate_limiting::RateLimitRule {
                                condition: match rule.condition {
                                    RateLimitRuleCondition::Header { headers } => {
                                        registry_v2::rate_limiting::RateLimitRuleCondition::Header(headers)
                                    }
                                    RateLimitRuleCondition::GraphqlOperation { operations } => {
                                        registry_v2::rate_limiting::RateLimitRuleCondition::GraphqlOperation(operations)
                                    }
                                    RateLimitRuleCondition::Ip { ips } => {
                                        registry_v2::rate_limiting::RateLimitRuleCondition::Ip(ips)
                                    }
                                    RateLimitRuleCondition::JwtClaim { jwt_claims } => {
                                        registry_v2::rate_limiting::RateLimitRuleCondition::JwtClaim(jwt_claims)
                                    }
                                },
                                name: rule.name,
                                limit: rule.limit,
                                duration: rule.duration,
                            })
                            .collect_vec(),
                    };
                }
                Err(_) => ctx.report_error(vec![directive.pos], "invalid syntax".to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::time::Duration;

    use indoc::indoc;

    use registry_v2::rate_limiting::{Header, Jwt, RateLimitConfig, RateLimitRule, RateLimitRuleCondition};

    #[test]
    fn empty_rules() {
        let config = indoc! {r#"
          extend schema
            @rateLimiting(
              rules: []
            )
        "#};

        let registry = crate::parse_registry(config).unwrap();

        let expected = RateLimitConfig { rules: vec![] };

        assert_eq!(expected, registry.rate_limiting);
    }

    #[test]
    fn complete_settings() {
        let config = indoc! {r#"
          extend schema
            @rateLimiting(
              rules: [
                {
                  name: "header",
                  condition: {
                    headers: [
                      { name: "my_header" },
                      { name: "my_header2", value: "not_used" }
                    ]
                  },
                  limit: 1000,
                  duration: 10
                },
                {
                  name: "ip",
                  condition: {
                    ips: ["0.0.0.0"]
                  },
                  limit: 1000,
                  duration: 10
                },
                {
                  name: "jwt",
                  condition: {
                    jwt_claims: [
                      { name: "my_claim" },
                      { name: "my_claim2", value: "string" },
                      { name: "my_claim3", value: ["array"] },
                      { name: "my_claim4", value: {} }
                    ]
                  },
                  limit: 1000,
                  duration: 10
                },
                {
                  name: "operations",
                  condition: {
                      operations: ["x", "y", "z"]
                  },
                  limit: 1000,
                  duration: 10
                },
              ]
            )
        "#};

        let registry = crate::parse_registry(config).unwrap();

        let expected = RateLimitConfig {
            rules: vec![
                RateLimitRule {
                    condition: RateLimitRuleCondition::Header(vec![
                        Header {
                            name: "my_header".to_string(),
                            value: None,
                        },
                        Header {
                            name: "my_header2".to_string(),
                            value: Some("not_used".to_string()),
                        },
                    ]),
                    name: "header".to_string(),
                    limit: 1000,
                    duration: Duration::from_secs(10),
                },
                RateLimitRule {
                    condition: RateLimitRuleCondition::Ip(vec![IpAddr::from_str("0.0.0.0").unwrap()]),
                    name: "ip".to_string(),
                    limit: 1000,
                    duration: Duration::from_secs(10),
                },
                RateLimitRule {
                    condition: RateLimitRuleCondition::JwtClaim(vec![
                        Jwt {
                            name: "my_claim".to_string(),
                            value: None,
                        },
                        Jwt {
                            name: "my_claim2".to_string(),
                            value: Some(serde_json::Value::String("string".to_string())),
                        },
                        Jwt {
                            name: "my_claim3".to_string(),
                            value: Some(serde_json::Value::Array(vec![serde_json::Value::String(
                                "array".to_string(),
                            )])),
                        },
                        Jwt {
                            name: "my_claim4".to_string(),
                            value: Some(serde_json::Value::Object(serde_json::Map::new())),
                        },
                    ]),
                    name: "jwt".to_string(),
                    limit: 1000,
                    duration: Duration::from_secs(10),
                },
                RateLimitRule {
                    condition: RateLimitRuleCondition::GraphqlOperation(vec![
                        "x".to_string(),
                        "y".to_string(),
                        "z".to_string(),
                    ]),
                    name: "operations".to_string(),
                    limit: 1000,
                    duration: Duration::from_secs(10),
                },
            ],
        };

        assert_eq!(expected, registry.rate_limiting);
    }

    #[test]
    fn invalid_input() {
        let config = indoc! {r#"
          extend schema
            @rateLimiting(
              rules: [
                {
                  name: "ip",
                  condition: {
                    ips: ["a"]
                  },
                  limit: 1000,
                  duration: 10
                },
              ]
            )
        "#};

        let result = crate::parse_registry(config);

        assert!(result.is_err_and(|err| err.to_string().contains("invalid syntax")))
    }

    #[test]
    fn invalid_input_2() {
        let config = indoc! {r#"
          extend schema
            @rateLimiting(
              rules: [
                {
                  name: "ip",
                  condition: {
                    random: ["a"]
                  },
                  limit: 1000,
                  duration: 10
                },
              ]
            )
        "#};

        let result = crate::parse_registry(config);

        assert!(result.is_err_and(|err| err.to_string().contains("invalid syntax")))
    }
}
