mod anonymous;
mod jwt;
mod v1;

use anonymous::AnonymousAuthorizer;
use futures_util::{future::BoxFuture, stream::FuturesOrdered, StreamExt};
use runtime::{auth::AccessToken, kv::KvStore, udf::AuthorizerInvoker};

pub trait Authorizer: Send + Sync + 'static {
    fn authorize<'a>(&'a self, headers: &'a http::HeaderMap) -> BoxFuture<'a, Option<AccessToken>>;
}

#[derive(Default)]
pub struct AuthService {
    authorizers: Vec<Box<dyn Authorizer>>,
}

impl AuthService {
    pub fn new_v1(config: config::v1::AuthConfig, kv: KvStore, udf_invoker: AuthorizerInvoker, ray_id: String) -> Self {
        Self {
            authorizers: vec![Box::new(v1::V1AuthProvider::new(ray_id, config, Some(kv), udf_invoker))],
        }
    }

    pub fn new_v2(config: config::v2::AuthConfig, kv: KvStore) -> Self {
        let authorizers: Vec<Box<dyn Authorizer>> = if config.providers.is_empty() {
            vec![Box::new(AnonymousAuthorizer)]
        } else {
            config
                .providers
                .into_iter()
                .map(|config| {
                    let authorizer: Box<dyn Authorizer> = match config {
                        config::v2::AuthProviderConfig::Jwt(config) => {
                            Box::new(jwt::JwtProvider::new(config, kv.clone()))
                        }
                    };
                    authorizer
                })
                .collect()
        };
        Self { authorizers }
    }

    pub async fn authorize(&self, headers: &http::HeaderMap) -> Option<AccessToken> {
        let fut = self
            .authorizers
            .iter()
            .map(|authorizer| authorizer.authorize(headers))
            .collect::<FuturesOrdered<_>>()
            .filter_map(|token| async move { token });
        futures_util::pin_mut!(fut);
        fut.next().await
    }

    pub fn with_first_authorizer(mut self, authorizer: impl Authorizer) -> Self {
        self.authorizers.insert(0, Box::new(authorizer));
        self
    }
}
