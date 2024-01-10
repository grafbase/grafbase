use config::latest::{AuthConfig, AuthProviderConfig};
use engine::RequestHeaders;
use futures_util::{stream::FuturesOrdered, StreamExt};
use jsonwebtoken::TokenData;
use runtime::kv::KvStore;
use std::collections::HashMap;

mod jwt;

pub enum AccessToken {
    Public,
    // boxing as clippy complains about enum size.
    Jwt(Box<TokenData<HashMap<String, serde_json::Value>>>),
}

#[async_trait::async_trait]
pub trait Authorizer: Send + Sync {
    async fn get_access_token(&self, headers: &RequestHeaders) -> Option<AccessToken>;
}

pub fn build(config: Option<&AuthConfig>, kv: &KvStore) -> Box<dyn Authorizer> {
    if let Some(config) = config {
        let authorizers = config
            .providers
            .iter()
            .map(|config| {
                let authorizer: Box<dyn Authorizer> = match config {
                    AuthProviderConfig::Jwt(config) => Box::new(jwt::JwtProvider::build(config, kv)),
                };
                authorizer
            })
            .collect::<Vec<_>>();
        match authorizers.len() {
            0 => Box::new(PublicAuthorizer),
            1 => authorizers.into_iter().next().expect("has one element"),
            _ => Box::new(MultiAuthorizer { authorizers }),
        }
    } else {
        Box::new(PublicAuthorizer)
    }
}

struct MultiAuthorizer {
    authorizers: Vec<Box<dyn Authorizer>>,
}

#[async_trait::async_trait]
impl Authorizer for MultiAuthorizer {
    async fn get_access_token(&self, headers: &RequestHeaders) -> Option<AccessToken> {
        let fut = self
            .authorizers
            .iter()
            .map(|authorizer| authorizer.get_access_token(headers))
            .collect::<FuturesOrdered<_>>()
            .filter_map(|token| async move { token });
        futures_util::pin_mut!(fut);
        fut.next().await
    }
}

struct PublicAuthorizer;

#[async_trait::async_trait]
impl Authorizer for PublicAuthorizer {
    async fn get_access_token(&self, _headers: &RequestHeaders) -> Option<AccessToken> {
        Some(AccessToken::Public)
    }
}
