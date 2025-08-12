use engine_error::{ErrorCode, ErrorResponse};
use futures::future::BoxFuture;
use runtime::extension::{ExtensionRequestContext, PublicMetadataEndpoint, Token};

use crate::{
    extension::{AuthenticationExtensionInstance, api::since_0_21_0::wit},
    resources::Headers,
};

impl AuthenticationExtensionInstance for super::ExtensionInstanceSince0_21_0 {
    fn authenticate<'a>(
        &'a mut self,
        ctx: &'a ExtensionRequestContext,
        headers: Headers,
    ) -> BoxFuture<'a, wasmtime::Result<Result<(Headers, Token), ErrorResponse>>> {
        Box::pin(async move {
            let resources = &mut self.store.data_mut().resources;
            let headers = resources.push(Headers::from(headers))?;
            let host_context = resources.push(wit::HostContext::from(ctx))?;
            let ctx = resources.push(wit::RequestContext::from(ctx))?;

            let result = self
                .inner
                .grafbase_sdk_authentication()
                .call_authenticate(&mut self.store, host_context, ctx, headers)
                .await?;

            let result = match result {
                Ok((headers, token)) => {
                    let headers = self.store.data_mut().resources.delete(headers)?;
                    Ok((headers, token.into()))
                }
                Err(err) => Err(self
                    .store
                    .data_mut()
                    .take_error_response(err, ErrorCode::Unauthenticated)?),
            };

            Ok(result)
        })
    }

    fn public_metadata(&mut self) -> BoxFuture<'_, wasmtime::Result<Result<Vec<PublicMetadataEndpoint>, String>>> {
        Box::pin(async move {
            let result = self
                .inner
                .grafbase_sdk_authentication()
                .call_public_metadata(&mut self.store)
                .await?;

            let result = match result {
                Ok(endpoints) => {
                    let store = self.store.data_mut();

                    let endpoints = endpoints
                        .into_iter()
                        .map(|public_metadata_endpoint| {
                            let headers = store
                                .resources
                                .delete(public_metadata_endpoint.response_headers)?
                                .into_inner()
                                .unwrap();

                            Ok(PublicMetadataEndpoint {
                                path: public_metadata_endpoint.path,
                                response_body: public_metadata_endpoint.response_body,
                                headers,
                            })
                        })
                        .collect::<wasmtime::Result<_>>()?;

                    Ok(endpoints)
                }
                Err(err) => Err(err.message),
            };

            Ok(result)
        })
    }
}
