use std::net::IpAddr;
use std::time::Duration;

#[derive(Debug, Default, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitConfig {
    pub rules: Vec<RateLimitRule>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RateLimitRule {
    pub condition: RateLimitRuleCondition,
    pub name: String,
    pub limit: u32,
    pub duration: Duration,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RateLimitRuleCondition {
    Header(Vec<Header>),
    GraphqlOperation(Vec<String>),
    Ip(Vec<IpAddr>),
    JwtClaim(Vec<Jwt>),
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Header {
    pub name: String,
    pub value: Option<String>,
}
#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Jwt {
    pub name: String,
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AnyOrSpecific<T> {
    Any,
    Specific(T)
}
