use engine_config::{AuthConfig, AuthProviderConfig};
use grafbase_workspace_hack as _;

mod anonymous;
mod jwt;

use anonymous::AnonymousAuthorizer;
use futures_util::{future::BoxFuture, stream::FuturesOrdered, StreamExt};
use runtime::{auth::AccessToken, kv::KvStore};
use tracing::{info_span, Instrument};

pub trait Authorizer: Send + Sync + 'static {
    fn get_access_token<'a>(&'a self, headers: &'a http::HeaderMap) -> BoxFuture<'a, Option<AccessToken>>;
}

#[derive(Default)]
pub struct AuthService {
    authorizers: Vec<Box<dyn Authorizer>>,
    only_anonymous: bool,
}

impl AuthService {
    pub fn new(authorizers: Vec<Box<dyn Authorizer>>) -> Self {
        Self {
            authorizers,
            only_anonymous: false,
        }
    }

    pub fn new_v2(config: AuthConfig, kv: KvStore) -> Self {
        if config.providers.is_empty() {
            Self {
                authorizers: vec![Box::new(AnonymousAuthorizer)],
                only_anonymous: true,
            }
        } else {
            let authorizers = config
                .providers
                .into_iter()
                .map(|config| {
                    let authorizer: Box<dyn Authorizer> = match config {
                        AuthProviderConfig::Jwt(config) => Box::new(jwt::JwtProvider::new(config, kv.clone())),
                        AuthProviderConfig::Anonymous => Box::new(AnonymousAuthorizer),
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

    pub fn with_first_authorizer(mut self, authorizer: impl Authorizer) -> Self {
        self.authorizers.insert(0, Box::new(authorizer));
        self
    }
}
