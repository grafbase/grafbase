mod anonymous;
mod jwt;

use anonymous::AnonymousAuthorizer;
use futures_util::{StreamExt, future::BoxFuture, stream::FuturesOrdered};
use runtime::{auth::AccessToken, kv::KvStore};
use schema::{AuthConfig, AuthProviderConfig};
use tracing::{Instrument, info_span};

pub trait Authorizer: Send + Sync + 'static {
    fn get_access_token<'a>(&'a self, headers: &'a http::HeaderMap) -> BoxFuture<'a, Option<AccessToken>>;
}

#[derive(Default)]
pub struct AuthService {
    authorizers: Vec<Box<dyn Authorizer>>,
    only_anonymous: bool,
}

impl AuthService {
    pub fn new(config: AuthConfig, kv: KvStore) -> Self {
        if config.providers.is_empty() {
            Self {
                authorizers: vec![Box::new(AnonymousAuthorizer)],
                only_anonymous: true,
            }
        } else {
            let authorizers = config
                .providers
                .into_iter()
                .flat_map(|config| {
                    let authorizer: Option<Box<dyn Authorizer>> = match config {
                        AuthProviderConfig::Jwt(config) => {
                            let authorizer = Box::new(jwt::JwtProvider::new(config, kv.clone()));
                            Some(authorizer)
                        }
                        AuthProviderConfig::Anonymous | AuthProviderConfig::Extension(_) => {
                            let authorizer = Box::new(AnonymousAuthorizer);
                            Some(authorizer)
                        }
                    };
                    authorizer
                })
                .collect();

            Self {
                authorizers,
                only_anonymous: false,
            }
        }
    }

    pub async fn authenticate(&self, headers: &http::HeaderMap) -> Option<AccessToken> {
        let fut = self
            .authorizers
            .iter()
            .map(|authorizer| authorizer.get_access_token(headers))
            .collect::<FuturesOrdered<_>>()
            .filter_map(|token| async move { token });

        futures_util::pin_mut!(fut);

        if self.only_anonymous {
            fut.next().await
        } else {
            let span = info_span!("authenticate");
            fut.next().instrument(span).await
        }
    }
}
