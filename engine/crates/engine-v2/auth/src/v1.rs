use std::collections::{BTreeMap, BTreeSet, HashMap};

use common_types::{auth::ExecutionAuth, UdfKind};
use config::v1::{AuthConfig, AuthProvider, AuthorizerProvider};
use futures_util::{future::BoxFuture, TryFutureExt};
use jwt_verifier::VerifiedToken;
use runtime::{
    auth::AccessToken,
    kv::KvStore,
    udf::{AuthorizerInvoker, UdfResponse},
};

use crate::Authorizer;

pub struct V1AuthProvider {
    ray_id: String,
    config: AuthConfig,
    jwks_cache: Option<KvStore>,
    udf_invoker: AuthorizerInvoker,
}

impl V1AuthProvider {
    pub fn new(
        // ray_id only matters in the cloud where we re-create the configuration each time. It
        // doesn't for local. And we will get rid of it with the tracing effort.
        ray_id: String,
        config: AuthConfig,
        jwks_cache: Option<KvStore>,
        udf_invoker: AuthorizerInvoker,
    ) -> Self {
        Self {
            ray_id,
            config,
            jwks_cache,
            udf_invoker,
        }
    }
}

impl Authorizer for V1AuthProvider {
    fn authorize<'a>(&'a self, headers: &'a http::HeaderMap) -> BoxFuture<'a, Option<AccessToken>> {
        Box::pin(async_runtime::make_send_on_wasm(self.get_access_token(headers)))
    }
}

impl V1AuthProvider {
    async fn get_access_token(&self, headers: &http::HeaderMap) -> Option<AccessToken> {
        let id_token = headers
            .get(http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer ").map(str::to_string));
        let result = match (id_token, &self.config.provider) {
            // API key has precedence over ID token
            (Some(token), Some(AuthProvider::Oidc(oidc_provider))) => {
                let client = jwt_verifier::Client {
                    trace_id: &self.ray_id,
                    jwks_cache: self.jwks_cache.clone(),
                    groups_claim: Some(&oidc_provider.groups_claim),
                    client_id: oidc_provider.client_id.as_deref(),
                    time_opts: Default::default(),
                    http_client: Default::default(),
                };
                client
                    .verify_rs_token_using_oidc_discovery(&token, &oidc_provider.issuer_base_url, &oidc_provider.issuer)
                    .inspect_err(|err| log::warn!(self.ray_id, "Unauthorized: {err:?}"))
                    .await
                    .ok()
                    .map(|verified_token| self.build_token_based_auth(verified_token))
            }
            (Some(token), Some(AuthProvider::Jwks(jwks_provider))) => {
                let client = jwt_verifier::Client {
                    trace_id: &self.ray_id,
                    jwks_cache: self.jwks_cache.clone(),
                    groups_claim: Some(&jwks_provider.groups_claim),
                    client_id: jwks_provider.client_id.as_deref(),
                    time_opts: Default::default(),
                    http_client: Default::default(),
                };

                client
                    .verify_rs_token_using_jwks_endpoint(
                        &token,
                        &jwks_provider.jwks_endpoint,
                        jwks_provider.issuer.as_deref(),
                    )
                    .inspect_err(|err| log::warn!(self.ray_id, "Unauthorized: {err:?}"))
                    .await
                    .ok()
                    .map(|verified_token| self.build_token_based_auth(verified_token))
            }
            (Some(token), Some(AuthProvider::Jwt(jwt_provider))) => {
                let client = jwt_verifier::Client {
                    trace_id: &self.ray_id,
                    groups_claim: Some(&jwt_provider.groups_claim),
                    client_id: jwt_provider.client_id.as_deref(),
                    jwks_cache: self.jwks_cache.clone(),
                    time_opts: Default::default(),
                    http_client: Default::default(),
                };

                client
                    .verify_hs_token(token, &jwt_provider.issuer, &jwt_provider.secret)
                    .map_err(|err| {
                        log::warn!(self.ray_id, "Unauthorized: {err:?}");
                        err
                    })
                    .ok()
                    .map(|verified_token| self.build_token_based_auth(verified_token))
            }
            (_, Some(AuthProvider::Authorizer(AuthorizerProvider { name }))) => {
                self.call_authorizer(name, headers).await
            }
            _ => Some(self.build_public_auth()),
        };
        log::debug!(
            self.ray_id,
            "Authorizing request using {:?} produces {result:?}",
            self.config
        );
        result.map(AccessToken::V1)
    }
}

impl V1AuthProvider {
    async fn call_authorizer(&self, name: &str, headers: &http::HeaderMap) -> Option<ExecutionAuth> {
        let request = runtime::udf::UdfRequest {
            name,
            request_id: &self.ray_id,
            udf_kind: UdfKind::Authorizer,
            payload: runtime::udf::AuthorizerRequestPayload {
                context: runtime::udf::UdfRequestContext {
                    request: runtime::udf::UdfRequestContextRequest {
                        headers: serde_json::to_value(
                            headers
                                .iter()
                                .filter_map(|(name, value)| Some((name.as_str(), value.to_str().ok()?)))
                                .collect::<HashMap<&str, &str>>(),
                        )
                        .expect("must be valid"),
                    },
                },
            },
        };
        let Ok(UdfResponse::Success(mut value)) =
            self.udf_invoker.invoke(&self.ray_id, request).await.inspect_err(|err| {
                log::warn!(self.ray_id, "authorizer failed: {err:?}");
            })
        else {
            return None;
        };

        log::trace!(self.ray_id, "Authorizer response: {value:?}");
        let Some(identity) = value.as_object_mut().and_then(|obj| obj.remove("identity")) else {
            // no identity returned, public access.
            return Some(self.build_public_auth());
        };

        let sub = match identity.get("sub") {
            Some(serde_json::Value::String(sub)) => Some(sub.clone()),
            None => None,
            other => {
                log::warn!(
                    self.ray_id,
                    "authorizer contract violation while getting subject, expected string, got {other:?}"
                );
                return None;
            }
        };
        let groups = match identity.get("groups") {
            Some(serde_json::Value::Array(groups)) => {
                let mut parsed_groups = BTreeSet::new();
                for group in groups {
                    let serde_json::Value::String(group) = group else {
                        return None;
                    };
                    parsed_groups.insert(group.clone());
                }
                parsed_groups
            }
            None => Default::default(),
            other => {
                log::warn!(
                    self.ray_id,
                    "authorizer contract violation while getting groups, expected array of strings, got {other:?}"
                );
                return None;
            }
        };

        let token_claims = serde_json::from_value::<BTreeMap<String, serde_json::Value>>(identity).ok()?;

        let verified_token = VerifiedToken {
            identity: sub,
            groups,
            token_claims,
        };
        log::debug!(self.ray_id, "Authorizer verified {verified_token:?}");
        Some(self.build_token_based_auth(verified_token))
    }

    fn build_token_based_auth(&self, verified_token: VerifiedToken) -> ExecutionAuth {
        // Get the global level group and owner based operations that are allowed.
        let private_public_and_group_ops = self.config.private_public_and_group_based_ops(&verified_token.groups);
        let allowed_owner_ops = self.config.owner_based_ops();

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
        ExecutionAuth::new_from_token(
            private_public_and_group_ops,
            verified_token.groups,
            subject_and_owner_ops,
            verified_token.token_claims,
        )
    }

    fn build_public_auth(&self) -> ExecutionAuth {
        ExecutionAuth::Public {
            global_ops: self.config.allowed_public_ops,
        }
    }
}
