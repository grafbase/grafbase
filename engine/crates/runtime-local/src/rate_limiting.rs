use std::net::IpAddr;
use std::num::NonZeroU32;
use std::str::FromStr;

use futures_util::future::{ready, BoxFuture};
use futures_util::FutureExt;
use governor::{DefaultKeyedRateLimiter, Quota};
use tungstenite::http;

use registry_v2::rate_limiting::{Header, Jwt, RateLimitRule, RateLimitRuleCondition};
use runtime::rate_limiting::{Error, RateLimiter, RateLimiterContext};

pub struct InMemoryRateLimiting {
    rate_limiters: Vec<(RateLimitRuleCondition, DefaultKeyedRateLimiter<String>)>,
}

impl InMemoryRateLimiting {
    pub fn new(rules: &[RateLimitRule]) -> Self {
        Self {
            rate_limiters: rules
                .iter()
                .map(|rule| {
                    let quota = rule
                        .limit
                        .checked_div(rule.duration.as_secs() as u32)
                        .expect("rate limiter with invalid per second quota");

                    (
                        rule.condition.clone(),
                        governor::RateLimiter::keyed(Quota::per_second(
                            NonZeroU32::new(quota).expect("rate limit duration cannot be 0"),
                        )),
                    )
                })
                .collect(),
        }
    }

    fn check_headers<'a>(
        &'a self,
        context: &(dyn RateLimiterContext + 'a),
        configured_headers: &[Header],
        rate_limiter: &DefaultKeyedRateLimiter<String>,
    ) -> Result<(), Error> {
        for configured_header in configured_headers {
            match context
                .header(
                    http::HeaderName::from_str(&configured_header.name)
                        .map_err(|err| Error::Internal(err.to_string()))?,
                )
                .zip(
                    configured_header
                        .value
                        .as_ref()
                        .and_then(|config_header_value| http::HeaderValue::from_str(config_header_value).ok()),
                ) {
                Some((request_header_value, configured_header_value))
                    if request_header_value.eq(&configured_header_value) =>
                {
                    // check the rate limiter
                    if let Ok(request_header_value) = request_header_value.to_str() {
                        if rate_limiter.check_key(&request_header_value.to_string()).is_err() {
                            return Err(Error::ExceededCapacity);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn check_operations<'a>(
        &'a self,
        context: &(dyn RateLimiterContext + 'a),
        configured_operations: &[String],
        rate_limiter: &DefaultKeyedRateLimiter<String>,
    ) -> Result<(), Error> {
        for configured_operation in configured_operations {
            if let Some(request_operation) = context.graphql_operation_name() {
                if request_operation == configured_operation
                    && rate_limiter.check_key(&request_operation.to_string()).is_err()
                {
                    return Err(Error::ExceededCapacity);
                }
            }
        }

        Ok(())
    }

    fn check_ips<'a>(
        &'a self,
        context: &(dyn RateLimiterContext + 'a),
        configured_ips: &[IpAddr],
        rate_limiter: &DefaultKeyedRateLimiter<String>,
    ) -> Result<(), Error> {
        for configured_ip in configured_ips {
            if let Some(request_ip) = context.ip() {
                if request_ip.eq(configured_ip) && rate_limiter.check_key(&configured_ip.to_string()).is_err() {
                    return Err(Error::ExceededCapacity);
                }
            }
        }

        Ok(())
    }

    fn check_jwt_claims<'a>(
        &'a self,
        context: &(dyn RateLimiterContext + 'a),
        configured_jwt_claims: &[Jwt],
        rate_limiter: &DefaultKeyedRateLimiter<String>,
    ) -> Result<(), Error> {
        for configured_jwt_claim in configured_jwt_claims {
            if let Some(request_jwt_claim) = context.jwt_claim(&configured_jwt_claim.name) {
                if let Some(configured_jwt_claim) = &configured_jwt_claim.value {
                    if request_jwt_claim.eq(configured_jwt_claim)
                        && rate_limiter.check_key(&request_jwt_claim.to_string()).is_err()
                    {
                        return Err(Error::ExceededCapacity);
                    }
                }
            }
        }

        Ok(())
    }
}

impl RateLimiter for InMemoryRateLimiting {
    fn limit<'a>(&'a self, context: Box<dyn RateLimiterContext + 'a>) -> BoxFuture<'a, Result<(), Error>> {
        for (condition, rate_limiter) in &self.rate_limiters {
            if let Err(err) = match condition {
                RateLimitRuleCondition::Header(headers) => self.check_headers(context.as_ref(), headers, rate_limiter),
                RateLimitRuleCondition::GraphqlOperation(operations) => {
                    self.check_operations(context.as_ref(), operations, rate_limiter)
                }
                RateLimitRuleCondition::Ip(ips) => self.check_ips(context.as_ref(), ips, rate_limiter),
                RateLimitRuleCondition::JwtClaim(claims) => {
                    self.check_jwt_claims(context.as_ref(), claims, rate_limiter)
                }
            } {
                return ready(Err(err)).boxed();
            };
        }

        ready(Ok(())).boxed()
    }
}
