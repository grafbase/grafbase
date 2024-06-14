use std::net::IpAddr;
use std::str::FromStr;

use http::{HeaderName, HeaderValue};
use serde_json::Value;

use runtime::auth;
use runtime::rate_limiting::RateLimiterContext;

pub struct RatelimitContext<'a> {
    auth: &'a auth::AccessToken,
    headers: &'a http::HeaderMap,
    request: &'a engine::Request,
}

impl<'a> RatelimitContext<'a> {
    pub fn new(request: &'a engine::Request, auth: &'a auth::AccessToken, headers: &'a http::HeaderMap) -> Self {
        Self { auth, headers, request }
    }
}

impl<'a> RateLimiterContext for RatelimitContext<'a> {
    fn header(&self, name: HeaderName) -> Option<&HeaderValue> {
        self.headers.get(name)
    }

    fn graphql_operation_name(&self) -> Option<&str> {
        self.request.operation_name()
    }

    fn ip(&self) -> Option<IpAddr> {
        self.headers
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').last())
            .and_then(|ip| IpAddr::from_str(ip).ok())
    }

    fn jwt_claim(&self, key: &str) -> &Value {
        self.auth.get_claim(key)
    }
}
