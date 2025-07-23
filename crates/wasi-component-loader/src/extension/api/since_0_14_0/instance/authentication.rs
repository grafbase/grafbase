use engine_error::ErrorResponse;
use futures::future::BoxFuture;
use runtime::extension::Token;

use crate::{
    WasmContext,
    extension::AuthenticationExtensionInstance,
    resources::{LegacyHeaders, OwnedOrShared},
};

impl AuthenticationExtensionInstance for super::ExtensionInstanceSince0_14_0 {
    fn authenticate(
        &mut self,
        _: &WasmContext,
        headers: OwnedOrShared<http::HeaderMap>,
    ) -> BoxFuture<'_, wasmtime::Result<Result<(OwnedOrShared<http::HeaderMap>, Token), ErrorResponse>>> {
        Box::pin(async move {
            let headers = self.store.data_mut().resources.push(LegacyHeaders::from(headers))?;
            let headers_rep = headers.rep();

            let result = self
                .inner
                .grafbase_sdk_authentication()
                .call_authenticate(&mut self.store, headers)
                .await?;

            let headers = self.store.data_mut().take_leased_resource(headers_rep)?;

            Ok(result
                .map(|token| (headers, token.into()))
                .map_err(|err| err.into_graphql_response(engine_error::ErrorCode::Unauthenticated)))
        })
    }
}
