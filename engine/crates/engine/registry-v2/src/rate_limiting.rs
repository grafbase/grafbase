use std::time::Duration;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitConfig {
    pub rules: Vec<RateLimitRule>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitRule {
    pub r#type: RateLimitRuleType,
    pub name: String,
    pub limit: u32,
    pub duration: Duration,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RateLimitRuleType {
    Header(Vec<String>),
    GraphqlOperationName(Vec<String>),
    Ip,
    JwtClaim(Vec<String>),
}
