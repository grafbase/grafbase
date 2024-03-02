use std::sync::Arc;

use common_types::auth::ExecutionAuth;
use futures_util::future::BoxFuture;
use gateway_core::{serving::X_API_KEY_HEADER, AdminAuthError};
use runtime::auth::AccessToken;

pub(crate) struct Authorizer;

#[async_trait::async_trait]
impl gateway_core::Authorizer for Authorizer {
    type Context = crate::Context;

    async fn authorize_admin_request(
        &self,
        _ctx: &Arc<Self::Context>,
        _request: &async_graphql::Request,
    ) -> Result<(), AdminAuthError> {
        Ok(())
    }
}

pub(crate) struct AnyApiKeyProvider;

impl gateway_v2_auth::Authorizer for AnyApiKeyProvider {
    fn get_access_token<'a>(&'a self, headers: &'a http::HeaderMap) -> BoxFuture<'a, Option<AccessToken>> {
        Box::pin(async { headers.get(X_API_KEY_HEADER).map(|_| ExecutionAuth::ApiKey.into()) })
    }
}
