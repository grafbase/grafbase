use anyhow::anyhow;
use futures::future::BoxFuture;
use runtime::extension::{AuthorizationDecisions, TokenRef};

use crate::{
    Error,
    extension::{
        AuthorizationExtensionInstance, QueryAuthorizationResult,
        api::wit::{QueryElements, ResponseElements},
    },
    resources::Lease,
};

impl AuthorizationExtensionInstance for super::ExtensionInstanceSince080 {
    fn authorize_query<'a>(
        &'a mut self,
        headers: Lease<http::HeaderMap>,
        _token: TokenRef<'a>,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, QueryAuthorizationResult> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let result = self
                .inner
                .grafbase_sdk_extension()
                .call_authorize_query(&mut self.store, elements.into())
                .await?;

            self.poisoned = false;

            result
                .map(|decisions| (headers, decisions.into(), Vec::new()))
                .map_err(Into::into)
        })
    }

    fn authorize_response<'a>(
        &'a mut self,
        _state: &'a [u8],
        _elements: ResponseElements<'a>,
    ) -> BoxFuture<'a, Result<AuthorizationDecisions, Error>> {
        Box::pin(async move { Err(anyhow!("authorize_response is not supported by sdk 0.8.*").into()) })
    }
}
