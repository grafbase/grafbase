mod anonymous;
mod jwt;

use std::sync::Arc;

use anonymous::AnonymousAuthorizer;
use error::{ErrorCode, ErrorResponse, GraphqlError};
use extension_catalog::ExtensionId;
use futures_util::{StreamExt, future::BoxFuture, stream::FuturesOrdered};
use gateway_config::{AuthenticationProvider, Config, DefaultAuthenticationBehavior};
use runtime::{authentication::LegacyToken, extension::GatewayExtensions, kv::KvStore};
use tracing::{Instrument, info_span};

trait LegacyAuthorizer: Send + Sync + 'static {
    fn get_access_token<'a>(&'a self, headers: &'a http::HeaderMap) -> BoxFuture<'a, Option<LegacyToken>>;
}

#[derive(Clone)]
pub struct AuthenticationService<Extensions>(Arc<AuthenticationServiceInner<Extensions>>);

impl<Extensions> std::ops::Deref for AuthenticationService<Extensions> {
    type Target = AuthenticationServiceInner<Extensions>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct AuthenticationServiceInner<Extensions> {
    extensions: Extensions,
    extension_ids: Option<Vec<ExtensionId>>,
    authorizers: Vec<Box<dyn LegacyAuthorizer>>,
    default_behavior: Option<DefaultAuthenticationBehavior>,
}

impl<Extensions> AuthenticationService<Extensions> {
    pub fn new(
        gateway_config: &Config,
        extensions: Extensions,
        extension_ids: Option<Vec<ExtensionId>>,
        default_behavior: Option<DefaultAuthenticationBehavior>,
        kv: &KvStore,
    ) -> Self {
        let authorizers = gateway_config
            .authentication
            .providers
            .iter()
            .map(|provider| {
                let authorizer: Box<dyn LegacyAuthorizer> = match provider {
                    AuthenticationProvider::Jwt(config) => Box::new(jwt::JwtProvider::new(config.clone(), kv.clone())),
                    AuthenticationProvider::Anonymous => Box::new(AnonymousAuthorizer),
                };
                authorizer
            })
            .collect::<Vec<_>>();

        Self(Arc::new(AuthenticationServiceInner {
            authorizers,
            extensions,
            extension_ids,
            default_behavior,
        }))
    }

    async fn legacy_authorizers(&self, headers: &http::HeaderMap) -> Option<LegacyToken> {
        let fut = self
            .authorizers
            .iter()
            .map(|authorizer| authorizer.get_access_token(headers))
            .collect::<FuturesOrdered<_>>()
            .filter_map(|token| async move { token });

        futures_util::pin_mut!(fut);

        let span = info_span!("authenticate");
        fut.next().instrument(span).await
    }
}

impl<Extensions: GatewayExtensions> runtime::authentication::Authenticate<Extensions::Context>
    for AuthenticationService<Extensions>
{
    async fn authenticate(
        &self,
        context: &Extensions::Context,
        headers: http::HeaderMap,
    ) -> Result<(http::HeaderMap, LegacyToken), ErrorResponse> {
        let (headers, result) = self
            .extensions
            .authenticate(context, headers, self.extension_ids.as_deref())
            .await;

        match result {
            None => {
                if self.authorizers.is_empty() {
                    return match self.default_behavior {
                        Some(DefaultAuthenticationBehavior::Anonymous) | None => Ok((headers, LegacyToken::Anonymous)),
                        _ => Err(unauthenticated()),
                    };
                }
                match self.legacy_authorizers(&headers).await {
                    Some(token) => Ok((headers, token)),
                    None => match self.default_behavior {
                        Some(DefaultAuthenticationBehavior::Anonymous) => Ok((headers, LegacyToken::Anonymous)),
                        _ => Err(unauthenticated()),
                    },
                }
            }
            Some(Ok(token)) => Ok((headers, LegacyToken::Extension(token))),
            Some(Err(error)) => {
                if self.authorizers.is_empty() {
                    return match self.default_behavior {
                        Some(DefaultAuthenticationBehavior::Anonymous) => Ok((headers, LegacyToken::Anonymous)),
                        _ => Err(error),
                    };
                }
                match self.legacy_authorizers(&headers).await {
                    Some(token) => Ok((headers, token)),
                    None => match self.default_behavior {
                        Some(DefaultAuthenticationBehavior::Anonymous) => Ok((headers, LegacyToken::Anonymous)),
                        _ => Err(error),
                    },
                }
            }
        }
    }
}

fn unauthenticated() -> ErrorResponse {
    ErrorResponse::new(http::StatusCode::UNAUTHORIZED)
        .with_error(GraphqlError::new("Unauthenticated", ErrorCode::Unauthenticated))
}
