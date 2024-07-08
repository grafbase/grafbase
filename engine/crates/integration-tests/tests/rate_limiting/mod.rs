use std::collections::{BTreeMap, HashSet};
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use futures::future::{ready, BoxFuture};
use http::HeaderMap;
use serde_json::json;
use tokio::time::Instant;

use engine::futures_util::FutureExt;
use gateway_core::ExecutionAuth;
use gateway_v2_auth::AuthService;
use integration_tests::engine_v1::{Error, GraphQlRequest};
use integration_tests::udfs::RustUdfs;
use integration_tests::{EngineBuilder, GatewayBuilder};
use registry_v2::rate_limiting::{AnyOr, Header, Jwt, RateLimitConfig, RateLimitRule, RateLimitRuleCondition};
use runtime::auth::AccessToken;
use runtime::udf::UdfResponse;

async fn gateway_builder() -> GatewayBuilder {
    let schema = r#"
            extend type Query {
                test: String! @resolver(name: "test")
            }
        "#;
    EngineBuilder::new(schema)
        .with_custom_resolvers(RustUdfs::new().resolver("test", UdfResponse::Success(json!("hello"))))
        .gateway_builder()
        .await
}

#[allow(clippy::panic)]
async fn expect_rate_limiting<'a, F>(f: F)
where
    F: Fn() -> BoxFuture<'a, Result<(Arc<engine::Response>, HeaderMap), Error>>,
{
    let destiny = Instant::now().checked_add(Duration::from_secs(60)).unwrap();

    loop {
        let response = Box::pin(f());
        let response = response.await;

        if matches!(response, Err(Error::Ratelimit(_))) {
            break;
        }

        if Instant::now().gt(&destiny) {
            panic!("Expected requests to get rate limited ...");
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn operations() {
    // prepare
    let query = "query Named { test }";
    let operation_name = "Named".to_string();
    let specific_header_gateway = gateway_builder()
        .await
        .with_rate_limiting_config(RateLimitConfig {
            rules: vec![RateLimitRule {
                name: "test".to_string(),
                condition: RateLimitRuleCondition::GraphqlOperation(AnyOr::Value(HashSet::from_iter([
                    operation_name.clone()
                ]))),
                limit: 10,
                duration: Duration::from_secs(10),
            }],
        })
        .build();

    let any_header_gateway = gateway_builder()
        .await
        .with_rate_limiting_config(RateLimitConfig {
            rules: vec![RateLimitRule {
                name: "test".to_string(),
                condition: RateLimitRuleCondition::GraphqlOperation(AnyOr::Any),
                limit: 10,
                duration: Duration::from_secs(10),
            }],
        })
        .build();

    // act && assert
    expect_rate_limiting(|| {
        async {
            let gql_request = GraphQlRequest {
                query: query.to_string(),
                operation_name: Some(operation_name.clone()),
                variables: None,
                extensions: None,
                doc_id: None,
            };
            specific_header_gateway.execute(gql_request).await
        }
        .boxed()
    })
    .await;

    expect_rate_limiting(|| {
        async {
            let gql_request = GraphQlRequest {
                query: query.to_string(),
                operation_name: Some(operation_name.clone()),
                variables: None,
                extensions: None,
                doc_id: None,
            };
            any_header_gateway.execute(gql_request).await
        }
        .boxed()
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn headers() {
    // prepare
    let query = "query Named { test }";
    let header = ("test-header".to_string(), "test".to_string());
    let specific_header_gateway = gateway_builder()
        .await
        .with_rate_limiting_config(RateLimitConfig {
            rules: vec![RateLimitRule {
                name: "test".to_string(),
                condition: RateLimitRuleCondition::Header(vec![Header {
                    name: header.0.clone(),
                    value: AnyOr::Value(HashSet::from_iter([header.1.clone()])),
                }]),
                limit: 10,
                duration: Duration::from_secs(10),
            }],
        })
        .build();

    let any_header_value_gateway = gateway_builder()
        .await
        .with_rate_limiting_config(RateLimitConfig {
            rules: vec![RateLimitRule {
                name: "test".to_string(),
                condition: RateLimitRuleCondition::Header(vec![Header {
                    name: header.0.clone(),
                    value: AnyOr::Value(HashSet::from_iter([header.1.clone()])),
                }]),
                limit: 10,
                duration: Duration::from_secs(10),
            }],
        })
        .build();

    // act && assert
    expect_rate_limiting(|| {
        async {
            specific_header_gateway
                .execute(query)
                .header(header.0.clone(), header.1.clone())
                .await
        }
        .boxed()
    })
    .await;
    expect_rate_limiting(|| {
        async {
            any_header_value_gateway
                .execute(query)
                .header(header.0.clone(), header.1.clone())
                .await
        }
        .boxed()
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn specific_ips() {
    // prepare
    let query = "query Named { test }";
    let ip = "1.1.1.1";
    let specific_ip_gateway = gateway_builder()
        .await
        .with_rate_limiting_config(RateLimitConfig {
            rules: vec![RateLimitRule {
                name: "test".to_string(),
                condition: RateLimitRuleCondition::Ip(AnyOr::Value(HashSet::from_iter(
                    [IpAddr::from_str(ip).unwrap()],
                ))),
                limit: 10,
                duration: Duration::from_secs(10),
            }],
        })
        .build();

    let any_ip_gateway = gateway_builder()
        .await
        .with_rate_limiting_config(RateLimitConfig {
            rules: vec![RateLimitRule {
                name: "test".to_string(),
                condition: RateLimitRuleCondition::Ip(AnyOr::Any),
                limit: 10,
                duration: Duration::from_secs(10),
            }],
        })
        .build();

    // act && assert
    expect_rate_limiting(|| async { specific_ip_gateway.execute(query).header("x-forwarded-for", ip).await }.boxed())
        .await;
    expect_rate_limiting(|| async { any_ip_gateway.execute(query).header("x-forwarded-for", ip).await }.boxed()).await;
}

#[tokio::test(flavor = "current_thread")]
async fn specific_jwt_claim() {
    // prepare
    let query = "query Named { test }";
    let jwt_claim = ("my_claim", "test");
    struct TestAuthorizer;
    impl gateway_v2_auth::Authorizer for TestAuthorizer {
        fn get_access_token<'a>(&'a self, _headers: &'a HeaderMap) -> BoxFuture<'a, Option<AccessToken>> {
            let mut claims = BTreeMap::new();
            claims.insert("my_claim".to_string(), serde_json::Value::String("test".to_string()));
            ready(Some(AccessToken::V1(ExecutionAuth::new_from_token(
                Default::default(),
                Default::default(),
                Default::default(),
                claims,
            ))))
            .boxed()
        }
    }

    let specific_jwt_claim_gateway = gateway_builder()
        .await
        .with_auth_service(AuthService::new(vec![Box::new(TestAuthorizer)]))
        .with_rate_limiting_config(RateLimitConfig {
            rules: vec![RateLimitRule {
                name: "test".to_string(),
                condition: RateLimitRuleCondition::JwtClaim(vec![Jwt {
                    name: jwt_claim.0.to_string(),
                    value: AnyOr::Value(serde_json::Value::String(jwt_claim.1.to_string())),
                }]),
                limit: 10,
                duration: Duration::from_secs(10),
            }],
        })
        .build();

    let any_jwt_claim_gateway = gateway_builder()
        .await
        .with_auth_service(AuthService::new(vec![Box::new(TestAuthorizer)]))
        .with_rate_limiting_config(RateLimitConfig {
            rules: vec![RateLimitRule {
                name: "test".to_string(),
                condition: RateLimitRuleCondition::JwtClaim(vec![Jwt {
                    name: jwt_claim.0.to_string(),
                    value: AnyOr::Value(serde_json::Value::String(jwt_claim.1.to_string())),
                }]),
                limit: 10,
                duration: Duration::from_secs(10),
            }],
        })
        .build();

    // act && assert
    expect_rate_limiting(|| async { specific_jwt_claim_gateway.execute(query).await }.boxed()).await;
    expect_rate_limiting(|| async { any_jwt_claim_gateway.execute(query).await }.boxed()).await;
}
