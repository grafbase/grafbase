use common_types::auth::ExecutionAuth;
use futures_util::future::BoxFuture;
use runtime::auth::AccessToken;

pub(crate) struct AnyApiKeyProvider;

impl gateway_v2_auth::Authorizer for AnyApiKeyProvider {
    fn authorize<'a>(&'a self, headers: &'a http::HeaderMap) -> BoxFuture<'a, Option<AccessToken>> {
        Box::pin(async { headers.get("x-api-key").map(|_| ExecutionAuth::ApiKey.into()) })
    }
}
