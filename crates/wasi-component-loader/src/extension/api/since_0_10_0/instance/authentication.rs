use futures::future::BoxFuture;
use runtime::extension::Token;

use crate::{
    ErrorResponse, SharedContext,
    extension::AuthenticationExtensionInstance,
    resources::{Headers, Lease},
};

impl AuthenticationExtensionInstance for super::ExtensionInstanceSince0_10_0 {
    fn authenticate(
        &mut self,
        _: SharedContext,
        headers: Lease<http::HeaderMap>,
    ) -> BoxFuture<'_, Result<(Lease<http::HeaderMap>, Token), ErrorResponse>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let headers = self.store.data_mut().push_resource(Headers::from(headers))?;
            let headers_rep = headers.rep();

            let result = self
                .inner
                .grafbase_sdk_authentication()
                .call_authenticate(&mut self.store, headers)
                .await?;

            let headers = self
                .store
                .data_mut()
                .take_resource::<Headers>(headers_rep)?
                .into_lease()
                .unwrap();

            self.poisoned = false;

            let token = result?;
            Ok((headers, token.into()))
        })
    }
}
