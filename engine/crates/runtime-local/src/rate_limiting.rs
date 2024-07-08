use std::collections::HashSet;
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::str::FromStr;

use futures_util::future::{ready, BoxFuture};
use futures_util::FutureExt;
use governor::{DefaultKeyedRateLimiter, Quota};
use tungstenite::http;

use registry_v2::rate_limiting::{AnyOr, Header, Jwt, RateLimitRule, RateLimitRuleCondition};
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
            if let Some(request_header_value) = context.header(
                http::HeaderName::from_str(&configured_header.name).map_err(|err| Error::Internal(err.to_string()))?,
            ) {
                let request_header_value = request_header_value
                    .to_str()
                    .map_err(|e| Error::Internal(e.to_string()))?
                    .to_string();

                match &configured_header.value {
                    AnyOr::Any => {
                        if rate_limiter.check_key(&request_header_value).is_err() {
                            return Err(Error::ExceededCapacity);
                        }
                    }
                    AnyOr::Value(specific_values) => {
                        if specific_values.contains(&request_header_value)
                            && rate_limiter.check_key(&request_header_value.to_string()).is_err()
                        {
                            return Err(Error::ExceededCapacity);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn check_operations<'a>(
        &'a self,
        context: &(dyn RateLimiterContext + 'a),
        configured_operations: &AnyOr<HashSet<String>>,
        rate_limiter: &DefaultKeyedRateLimiter<String>,
    ) -> Result<(), Error> {
        if let Some(request_operation) = context.graphql_operation_name() {
            match configured_operations {
                AnyOr::Any => {
                    if rate_limiter.check_key(&request_operation.to_string()).is_err() {
                        return Err(Error::ExceededCapacity);
                    }
                }
                AnyOr::Value(configured_operations) => {
                    if configured_operations.contains(request_operation)
                        && rate_limiter.check_key(&request_operation.to_string()).is_err()
                    {
                        return Err(Error::ExceededCapacity);
                    }
                }
            }
        }

        Ok(())
    }

    fn check_ips<'a>(
        &'a self,
        context: &(dyn RateLimiterContext + 'a),
        configured_ips: &AnyOr<HashSet<IpAddr>>,
        rate_limiter: &DefaultKeyedRateLimiter<String>,
    ) -> Result<(), Error> {
        if let Some(request_ip) = context.ip() {
            match configured_ips {
                AnyOr::Any => {
                    if rate_limiter.check_key(&request_ip.to_string()).is_err() {
                        return Err(Error::ExceededCapacity);
                    }
                }
                AnyOr::Value(configured_ips) => {
                    if configured_ips.contains(&request_ip) && rate_limiter.check_key(&request_ip.to_string()).is_err()
                    {
                        return Err(Error::ExceededCapacity);
                    }
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
                match &configured_jwt_claim.value {
                    AnyOr::Any => {
                        if rate_limiter.check_key(&request_jwt_claim.to_string()).is_err() {
                            return Err(Error::ExceededCapacity);
                        }
                    }
                    AnyOr::Value(claim) => {
                        if claim.eq(request_jwt_claim)
                            && rate_limiter.check_key(&request_jwt_claim.to_string()).is_err()
                        {
                            return Err(Error::ExceededCapacity);
                        }
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
