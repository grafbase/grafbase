mod api_key;

use std::collections::BTreeMap;

use common_types::{auth::ExecutionAuth, UdfKind};
use engine::{AuthConfig, AuthorizerProvider};
use futures_util::TryFutureExt;
use jwt_verifier::{VerificationError, VerifiedToken};
use runtime::udf::{AuthorizerRequestPayload, CustomResolverResponse, UdfInvoker};
use worker::{Env, Request};
use worker_utils::RequestExt;

pub use self::api_key::*;
use crate::platform::context::RequestContext;

const AUTHORIZATION_HEADER: &str = "authorization";
pub const X_API_KEY_HEADER: &str = "x-api-key";

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum AuthError {
    #[error("invalid api key: {0}")]
    #[cfg(not(feature = "local"))]
    APIKeyVerificationError(#[from] APIKeyVerificationError),
    #[error("verification error: {0}")]
    VerificationError(#[from] VerificationError),
    #[error("bindgen error: {0}")]
    BindgenError(#[from] worker::Error),
    #[error("authorizer invocation error")]
    UdfError,
    #[error("authorizer returned invalid token claims: {0}")]
    InvalidTokenClaims(String),
}

#[derive(derive_more::Debug)]
pub struct AuthResponse {
    #[debug(skip)]
    pub gql_request: engine::Request,
    pub auth: ExecutionAuth,
}

impl AuthResponse {
    fn token_based(verified_token: VerifiedToken, gql_request: engine::Request, auth_config: &AuthConfig) -> Self {
        // Get the global level group and owner based operations that are allowed.
        let private_public_and_group_ops = auth_config.private_public_and_group_based_ops(&verified_token.groups);
        let allowed_owner_ops = auth_config.owner_based_ops();

        // It's fine for ops and groups to be empty as there might
        // be some model-level auth rule evaluated later
        let subject_and_owner_ops = verified_token.identity.and_then(|subject| {
            // Turn off owner-based auth if no operations are allowed.
            if allowed_owner_ops.is_empty() {
                None
            } else {
                Some((subject, allowed_owner_ops))
            }
        });
        let auth = ExecutionAuth::new_from_token(
            private_public_and_group_ops,
            verified_token.groups,
            subject_and_owner_ops,
            verified_token.token_claims,
        );
        Self { gql_request, auth }
    }

    fn public(gql_request: engine::Request, auth_config: &AuthConfig) -> Self {
        let auth = ExecutionAuth::Public {
            global_ops: auth_config.allowed_public_ops,
        };
        Self { gql_request, auth }
    }
}

#[allow(clippy::panic)]
pub async fn authorize_request(
    req: &Request,
    gql_request: engine::Request,
    env: &Env,
    request_context: &RequestContext,
) -> Result<AuthResponse, AuthError> {
    let auth_config = &request_context
        .config
        .customer_deployment_config
        .common_customer_deployment_config()
        .auth_config;
    let api_key = req.header_or_query_param(X_API_KEY_HEADER);
    let id_token = req
        .header_or_query_param(AUTHORIZATION_HEADER)
        .and_then(|val| val.strip_prefix("Bearer ").map(str::to_string));

    let ray_id = &request_context.cloudflare_request_context.ray_id;

    let result = match (api_key, id_token, &auth_config.provider) {
        // API key has precedence over ID token
        (Some(api_key), _, _) => {
            cfg_if::cfg_if! {
                if #[cfg(feature = "local")] {
                    // Grant full access if any API key is passed locally
                    let _ = api_key;
                    log::warn!(
                        ray_id,
                        "Ignoring API key verification locally"
                    );
                    AuthResponse{
                        gql_request,
                        auth: ExecutionAuth::new_from_api_keys(),
                    }
                } else {
                    use tracing::Instrument;
                    request_context
                        .api_key_auth
                        .verify_api_key(&api_key, env)
                        .inspect_err(|err| log::warn!(ray_id, "Unauthorized: {err:?}"))
                        .instrument(tracing::info_span!("authorization"))
                        .await
                        .map(|_allowed_ops| AuthResponse {
                            gql_request,
                            auth: ExecutionAuth::new_from_api_keys(),
                        })?
                }
            }
        }
        (None, Some(token), Some(engine::AuthProvider::Oidc(oidc_provider))) => {
            log::debug!(
                ray_id,
                "Verifying ID token {token} using OIDC provider {oidc_provider:?}"
            );
            let client = jwt_verifier::Client {
                trace_id: ray_id,
                jwks_cache: get_jwks_cache(env),
                groups_claim: Some(&oidc_provider.groups_claim),
                client_id: oidc_provider.client_id.as_deref(),
                ..Default::default()
            };
            client
                .verify_rs_token_using_oidc_discovery(&token, &oidc_provider.issuer_base_url, &oidc_provider.issuer)
                .inspect_err(|err| log::warn!(ray_id, "Unauthorized: {err:?}"))
                .await
                .map(|verified_token| AuthResponse::token_based(verified_token, gql_request, auth_config))?
        }
        (None, Some(token), Some(engine::AuthProvider::Jwks(jwks_provider))) => {
            log::debug!(
                ray_id,
                "Verifying ID token {token} using JWKS provider {jwks_provider:?}"
            );
            let client = jwt_verifier::Client {
                trace_id: ray_id,
                jwks_cache: get_jwks_cache(env),
                groups_claim: Some(&jwks_provider.groups_claim),
                client_id: jwks_provider.client_id.as_deref(),
                ..Default::default()
            };

            client
                .verify_rs_token_using_jwks_endpoint(
                    &token,
                    &jwks_provider.jwks_endpoint,
                    jwks_provider.issuer.as_deref(),
                )
                .inspect_err(|err| log::warn!(ray_id, "Unauthorized: {err:?}"))
                .await
                .map(|verified_token| AuthResponse::token_based(verified_token, gql_request, auth_config))?
        }
        (None, Some(token), Some(engine::AuthProvider::Jwt(jwt_provider))) => {
            log::debug!(ray_id, "Verifying ID token {token} using JWT provider {jwt_provider:?}");

            let client = jwt_verifier::Client {
                trace_id: ray_id,
                groups_claim: Some(&jwt_provider.groups_claim),
                client_id: jwt_provider.client_id.as_deref(),
                ..Default::default()
            };

            client
                .verify_hs_token(&token, &jwt_provider.issuer, &jwt_provider.secret)
                .map_err(|err| {
                    log::warn!(ray_id, "Unauthorized: {err:?}");
                    err
                })
                .map(|verified_token| AuthResponse::token_based(verified_token, gql_request, auth_config))?
        }
        (None, _, Some(engine::AuthProvider::Authorizer(AuthorizerProvider { name }))) => {
            cfg_if::cfg_if! {
                if #[cfg(feature = "local")] {
                    let bridge_port = worker_env::EnvExt::var_get(
                        env,
                        worker_env::VarType::Var,
                        gateway_adapter_local::execution::BRIDGE_PORT_ENV_VAR,
                    )
                    .unwrap_or_else(|_| panic!("Missing env var {}",
                        gateway_adapter_local::execution::BRIDGE_PORT_ENV_VAR));
                    let invoker = runtime_local::UdfInvokerImpl::new(runtime_local::Bridge::new(bridge_port));
                    call_authorizer(ray_id, req.headers().entries().collect(), name.clone(), invoker, gql_request, auth_config).await?
                } else {
                    let dispatcher = env.dynamic_dispatcher("dispatcher")?;
                    let udf_workers = request_context.config.customer_deployment_config.common_customer_deployment_config().udf_bindings.clone();
                    let invoker = grafbase_cloud::udf_invoker::UdfInvokerImpl::new(dispatcher, udf_workers);
                    call_authorizer(ray_id, req.headers().entries().collect(), name.clone(), invoker, gql_request, auth_config).await?
                }
            }
        }
        (None, None, _) | (None, _, None) => AuthResponse::public(gql_request, auth_config),
    };
    log::debug!(ray_id, "Authorizing request using {auth_config:?} produces {result:?}");
    Ok(result)
}

async fn call_authorizer<Invoker>(
    ray_id: &str,
    headers: std::collections::HashMap<String, String>,
    name: String,
    invoker: Invoker,
    gql_request: engine::Request,
    auth_config: &AuthConfig,
) -> Result<AuthResponse, AuthError>
where
    Invoker: UdfInvoker<AuthorizerRequestPayload>,
{
    let request = runtime::udf::UdfRequest {
        name: &name,
        request_id: ray_id,
        udf_kind: UdfKind::Authorizer,
        payload: runtime::udf::AuthorizerRequestPayload {
            context: runtime::udf::UdfRequestContext {
                request: runtime::udf::UdfRequestContextRequest {
                    headers: serde_json::to_value(&headers).expect("must be valid"),
                },
            },
        },
    };
    let value = UdfInvoker::invoke(&invoker, ray_id, request)
        .map_err(|err| {
            log::warn!(ray_id, "authorizer failed: {err:?}");
            AuthError::VerificationError(VerificationError::Authorizer)
        })
        .and_then(|response| {
            futures_util::future::ready(match response {
                CustomResolverResponse::Success(value) => Ok(value),
                CustomResolverResponse::GraphQLError { .. } | CustomResolverResponse::Error(_) => {
                    Err(AuthError::UdfError)
                }
            })
        })
        .await?;

    log::trace!(ray_id, "Authorizer respose: {value:?}");
    if let Some(identity) = value.get("identity") {
        let sub = match identity.get("sub") {
            Some(serde_json::Value::String(sub)) => Ok(Some(sub.clone())),
            None => Ok(None),
            other => {
                log::warn!(
                    ray_id,
                    "authorizer contract violation while getting subject, expected string, got {other:?}"
                );
                Err(AuthError::VerificationError(VerificationError::Authorizer))
            }
        }?;
        let groups = match identity.get("groups") {
            Some(serde_json::Value::Array(groups)) => groups
                .iter()
                .map(|val| match val {
                    serde_json::Value::String(val) => Ok(val.clone()),
                    other => {
                        log::warn!(
                            ray_id,
                            "authorizer contract violation while getting groups, expected string, got {other:?}"
                        );
                        Err(AuthError::VerificationError(VerificationError::Authorizer))
                    }
                })
                .collect::<Result<_, _>>(),
            None => Ok(Default::default()),
            other => {
                log::warn!(
                    ray_id,
                    "authorizer contract violation while getting groups, expected array of strings, got {other:?}"
                );
                Err(AuthError::VerificationError(VerificationError::Authorizer))
            }
        }?;

        let token_claims = serde_json::from_value::<BTreeMap<String, serde_json::Value>>(identity.clone())
            .map_err(|_| AuthError::InvalidTokenClaims(identity.to_string()))?;

        let verified_token = VerifiedToken {
            identity: sub,
            groups,
            token_claims,
        };
        log::debug!(ray_id, "Authorizer verified {verified_token:?}");
        Ok(AuthResponse::token_based(verified_token, gql_request, auth_config))
    } else {
        // no identity returned, public access.
        Ok(AuthResponse::public(gql_request, auth_config))
    }
}

#[cfg(feature = "local")]
fn get_jwks_cache(_env: &Env) -> Option<worker::kv::KvStore> {
    None
}

#[cfg(not(feature = "local"))]
#[allow(clippy::unnecessary_wraps)]
fn get_jwks_cache(env: &Env) -> Option<worker::kv::KvStore> {
    const JWKS_CACHE_KV_NAMESPACE: &str = "JWKS_CACHE";
    Some(env.kv(JWKS_CACHE_KV_NAMESPACE).expect("the KV namespace must exist"))
}
