mod anonymous;
mod jwt;

use anonymous::AnonymousAuthorizer;
use error::{ErrorCode, ErrorResponse, GraphqlError};
use extension_catalog::{ExtensionCatalog, ExtensionId};
use futures_util::{StreamExt, future::BoxFuture, stream::FuturesOrdered};
use gateway_config::{AuthenticationProvider, DefaultAuthenticationBehavior};
use runtime::{authentication::LegacyToken, extension::ExtensionRuntime, kv::KvStore};
use tracing::{Instrument, info_span};

trait LegacyAuthorizer: Send + Sync + 'static {
    fn get_access_token<'a>(&'a self, headers: &'a http::HeaderMap) -> BoxFuture<'a, Option<LegacyToken>>;
}

pub struct AuthenticationService<Extensions> {
    extensions: Extensions,
    authentication_extension_ids: Vec<ExtensionId>,
    authorizers: Vec<Box<dyn LegacyAuthorizer>>,
    default_token: Option<LegacyToken>,
}

impl<Extensions> AuthenticationService<Extensions> {
    pub fn new(
        config: &gateway_config::Config,
        catalog: &ExtensionCatalog,
        extensions: Extensions,
        kv: &KvStore,
    ) -> Self {
        let authorizers = config
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

        let authentication_extension_ids = catalog
            .iter_with_id()
            .filter_map(|(id, extension)| match extension.manifest.kind {
                extension_catalog::Kind::Authentication(_) => Some(id),
                _ => None,
            })
            .collect::<Vec<_>>();

        let default_token = match config.authentication.default {
            Some(DefaultAuthenticationBehavior::Anonymous) => Some(LegacyToken::Anonymous),
            Some(DefaultAuthenticationBehavior::Deny) => None,
            None => {
                if !authorizers.is_empty() || !authentication_extension_ids.is_empty() {
                    None
                } else {
                    Some(LegacyToken::Anonymous)
                }
            }
        };
        Self {
            authorizers,
            extensions,
            authentication_extension_ids,
            default_token,
        }
    }

    async fn legacy_authorizers(&self, headers: &http::HeaderMap) -> Option<LegacyToken> {
        if self.authorizers.is_empty() {
            return None;
        }
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

impl<Extensions: ExtensionRuntime> runtime::authentication::Authenticate for AuthenticationService<Extensions> {
    async fn authenticate(&self, headers: http::HeaderMap) -> Result<(http::HeaderMap, LegacyToken), ErrorResponse> {
        if !self.authentication_extension_ids.is_empty() {
            let (headers, result) = self
                .extensions
                .authenticate(&self.authentication_extension_ids, headers)
                .await;
            match result {
                Ok(token) => Ok((headers, LegacyToken::Extension(token))),
                Err(error) => match self.legacy_authorizers(&headers).await {
                    Some(token) => Ok((headers, token)),
                    None => match self.default_token.clone() {
                        Some(token) => Ok((headers, token)),
                        None => Err(error),
                    },
                },
            }
        } else {
            match self.legacy_authorizers(&headers).await {
                Some(token) => Ok((headers, token)),
                None => match self.default_token.clone() {
                    Some(token) => Ok((headers, token)),
                    None => Err(unauthenticated()),
                },
            }
        }
    }
}

fn unauthenticated() -> ErrorResponse {
    ErrorResponse::new(http::StatusCode::UNAUTHORIZED)
        .with_error(GraphqlError::new("Unauthenticated", ErrorCode::Unauthenticated))
}
