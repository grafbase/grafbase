use engine_error::{ErrorCode, GraphqlError};
use futures::future::BoxFuture;
use runtime::extension::{AuthorizationDecisions, TokenRef};

use crate::{
    WasmContext,
    extension::{
        AuthorizationExtensionInstance, QueryAuthorizationResult,
        api::wit::{QueryElements, ResponseElements, TokenParam},
    },
    resources::{LegacyHeaders, OwnedOrShared},
};

impl AuthorizationExtensionInstance for super::ExtensionInstanceSince0_16_0 {
    fn authorize_query<'a>(
        &'a mut self,
        _: &'a WasmContext,
        headers: OwnedOrShared<http::HeaderMap>,
        token: TokenRef<'a>,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, QueryAuthorizationResult> {
        Box::pin(async move {
            let headers = self.store.data_mut().resources.push(LegacyHeaders::from(headers))?;
            let headers_rep = headers.rep();

            let token_param = token.as_bytes().map(TokenParam::Bytes).unwrap_or(TokenParam::Anonymous);
            let result = self
                .inner
                .grafbase_sdk_authorization()
                .call_authorize_query(&mut self.store, headers, token_param, elements.into())
                .await?;

            let headers = self.store.data_mut().take_leased_resource(headers_rep)?;

            let result = match result {
                Ok((decisions, state)) => Ok((headers, decisions.into(), state)),
                Err(err) => Err(self
                    .store
                    .data_mut()
                    .take_error_response_sdk17(err.into(), ErrorCode::Unauthorized)?),
            };

            Ok(result)
        })
    }

    fn authorize_response<'a>(
        &'a mut self,
        _: &'a WasmContext,
        state: &'a [u8],
        elements: ResponseElements<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<AuthorizationDecisions, GraphqlError>>> {
        Box::pin(async move {
            let result = self
                .inner
                .grafbase_sdk_authorization()
                .call_authorize_response(&mut self.store, state, elements)
                .await?;

            Ok(result
                .map(Into::into)
                .map_err(|err| err.into_graphql_error(ErrorCode::Unauthorized)))
        })
    }
}
