use futures::future::BoxFuture;
use runtime::extension::Token;

use crate::{
    ErrorResponse, SharedContext,
    extension::AuthenticationExtensionInstance,
    resources::{Headers, Lease},
};

impl AuthenticationExtensionInstance for super::ExtensionInstanceSince0_17_0 {
    fn authenticate(
        &mut self,
        context: SharedContext,
        headers: Lease<http::HeaderMap>,
    ) -> BoxFuture<'_, Result<(Lease<http::HeaderMap>, Token), ErrorResponse>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let headers = self.store.data_mut().push_resource(Headers::from(headers))?;
            let headers_rep = headers.rep();

            let context = self.store.data_mut().push_resource(context)?;

            let result = self
                .inner
                .grafbase_sdk_authentication()
                .call_authenticate(&mut self.store, context, headers)
                .await?;

            let headers = self
                .store
                .data_mut()
                .take_resource::<Headers>(headers_rep)?
                .into_lease()
                .unwrap();

            self.poisoned = false;

            let token = result.map_err(|err| super::error_response_from_wit(&mut self.store, err))?;

            Ok((headers, token.into()))
        })
    }

    fn public_metadata(
        &mut self,
    ) -> BoxFuture<'_, Result<Vec<runtime::authentication::PublicMetadataEndpoint>, crate::Error>> {
        Box::pin(async move {
            let result = self
                .inner
                .grafbase_sdk_authentication()
                .call_public_metadata(&mut self.store)
                .await??;

            let store = self.store.data_mut();

            let endpoints = result
                .into_iter()
                .map(|public_metadata_endpoint| {
                    let headers = store
                        .take_resource::<Headers>(public_metadata_endpoint.response_headers.rep())?
                        .into_inner()
                        .unwrap();

                    crate::Result::<_>::Ok(runtime::authentication::PublicMetadataEndpoint {
                        path: public_metadata_endpoint.path,
                        response_body: public_metadata_endpoint.response_body,
                        headers,
                    })
                })
                .collect::<Result<_, _>>()?;

            Ok(endpoints)
        })
    }
}
