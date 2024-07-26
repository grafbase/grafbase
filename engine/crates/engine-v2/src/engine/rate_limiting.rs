use std::net::IpAddr;

use config::GLOBAL_RATE_LIMIT_KEY;
use serde_json::Value;

use runtime::rate_limiting::RateLimiterContext;

pub enum RateLimitContext<'a> {
    Global,
    Subgraph(&'a str),
}

impl RateLimiterContext for RateLimitContext<'_> {
    fn header(&self, _name: http::HeaderName) -> Option<&http::HeaderValue> {
        None
    }

    fn graphql_operation_name(&self) -> Option<&str> {
        None
    }

    fn ip(&self) -> Option<IpAddr> {
        None
    }

    fn jwt_claim(&self, _key: &str) -> Option<&Value> {
        None
    }

    fn key(&self) -> Option<&str> {
        Some(match self {
            RateLimitContext::Global => GLOBAL_RATE_LIMIT_KEY,
            RateLimitContext::Subgraph(name) => name,
        })
    }

    fn is_global(&self) -> bool {
        matches!(self, Self::Global)
    }
}
