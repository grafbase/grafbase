use grafbase_workspace_hack as _;

mod anonymous;
mod jwt;
mod v1;

use anonymous::AnonymousAuthorizer;
use futures_util::{future::BoxFuture, stream::FuturesOrdered, StreamExt};
use runtime::{auth::AccessToken, kv::KvStore, udf::AuthorizerInvoker};
use tracing::instrument;

pub trait Authorizer: Send + Sync + 'static {
    fn get_access_token<'a>(&'a self, headers: &'a http::HeaderMap) -> BoxFuture<'a, Option<AccessToken>>;
}

#[derive(Default)]
/// A service responsible for handling authentication using multiple authorizers.
pub struct AuthService {
    authorizers: Vec<Box<dyn Authorizer>>,
}

impl AuthService {
    /// Creates a new instance of `AuthService` with the specified authorizers.
    ///
    /// # Parameters
    ///
    /// - `authorizers`: A vector of authorizers that implement the `Authorizer` trait.
    ///
    /// # Returns
    ///
    /// A new instance of `AuthService` configured with the provided authorizers.
    pub fn new(authorizers: Vec<Box<dyn Authorizer>>) -> Self {
        Self { authorizers }
    }

    /// Creates a new instance of `AuthService` for standalone graph authentication.
    ///
    /// # Parameters
    ///
    /// - `config`: Configuration specific to standalone graph authentication.
    /// - `kv`: Key-value store used for storing and retrieving authentication data.
    /// - `udf_invoker`: An invoker for User Defined Functions related to authorization.
    /// - `ray_id`: A unique (Cloudflare) identifier for the request.
    ///
    /// # Returns
    ///
    /// A new instance of `AuthService` configured for standalone graph authentication.
    pub fn new_v1(config: config::v1::AuthConfig, kv: KvStore, udf_invoker: AuthorizerInvoker, ray_id: String) -> Self {
        Self {
            authorizers: vec![Box::new(v1::V1AuthProvider::new(ray_id, config, Some(kv), udf_invoker))],
        }
    }

    /// Creates a new instance of `AuthService` for federation authentication using the specified
    /// configuration and key-value store.
    ///
    /// # Parameters
    ///
    /// - `config`: Configuration specific to the federation authentication providers.
    /// - `kv`: Cloudflare key-value store used for storing and retrieving authentication data.
    ///
    /// # Returns
    ///
    /// A new instance of `AuthService` configured for federation authentication.
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
                        config::v2::AuthProviderConfig::Anonymous => Box::new(AnonymousAuthorizer),
                    };
                    authorizer
                })
                .collect()
        };
        Self { authorizers }
    }

    #[instrument(skip_all)]
    /// Authenticates a given set of headers using the registered authorizers.
    ///
    /// This method iterates over all configured authorizers and attempts to retrieve an access token
    /// from each one in order. The first successful retrieval will be returned.
    ///
    /// # Parameters
    ///
    /// - `headers`: A reference to the HTTP headers that may contain authentication information.
    ///
    /// # Returns
    ///
    /// An `Option<AccessToken>`, where `Some(AccessToken)` represents a successfully retrieved token,
    /// and `None` indicates that no valid token could be obtained from any of the authorizers.
    pub async fn authenticate(&self, headers: &http::HeaderMap) -> Option<AccessToken> {
        let fut = self
            .authorizers
            .iter()
            .map(|authorizer| authorizer.get_access_token(headers))
            .collect::<FuturesOrdered<_>>()
            .filter_map(|token| async move { token });

        futures_util::pin_mut!(fut);

        fut.next().await
    }

    /// Adds a new authorizer to the beginning of the authorizers list.
    ///
    /// This method allows you to specify a custom authorizer that will take precedence
    /// over the existing ones during authentication attempts. The new authorizer is
    /// inserted at the front of the list, ensuring it is checked first.
    ///
    /// # Parameters
    ///
    /// - `authorizer`: An instance of a type that implements the `Authorizer` trait.
    ///
    /// # Returns
    ///
    /// The updated instance of `AuthService` with the new authorizer added.
    pub fn with_first_authorizer(mut self, authorizer: impl Authorizer) -> Self {
        self.authorizers.insert(0, Box::new(authorizer));
        self
    }
}
