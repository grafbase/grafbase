use engine_error::{ErrorCode, GraphqlError};
use futures::future::BoxFuture;
use runtime::extension::{AuthorizationDecisions, TokenRef};

use crate::{
    WasmContext,
    extension::{
        AuthorizationExtensionInstance, QueryAuthorizationResult,
        api::{
            since_0_19_0::wit::exports::grafbase::sdk::authorization::AuthorizationOutput,
            wit::{Headers, QueryElements, ResponseElements, TokenParam},
        },
    },
};

impl AuthorizationExtensionInstance for super::ExtensionInstanceSince0_19_0 {
    fn authorize_query<'a>(
        &'a mut self,
        context: &'a WasmContext,
        headers: Headers,
        token: TokenRef<'a>,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, QueryAuthorizationResult> {
        Box::pin(async move {
            let context = self.store.data_mut().resources.push(context.clone())?;
            let headers = self.store.data_mut().resources.push(headers)?;

            let token_param = token.as_bytes().map(TokenParam::Bytes).unwrap_or(TokenParam::Anonymous);

            let result = self
                .inner
                .grafbase_sdk_authorization()
                .call_authorize_query(&mut self.store, context, headers, token_param, elements)
                .await?;

            let result = match result {
                Ok(AuthorizationOutput {
                    decisions,
                    state,
                    headers,
                }) => {
                    let headers = self.store.data_mut().resources.delete(headers)?;
                    Ok((headers, decisions.into(), state))
                }
                Err(err) => Err(self
                    .store
                    .data_mut()
                    .take_error_response(err, ErrorCode::Unauthorized)?),
            };

            Ok(result)
        })
    }

    fn authorize_response<'a>(
        &'a mut self,
        context: &'a WasmContext,
        state: &'a [u8],
        elements: ResponseElements<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<AuthorizationDecisions, GraphqlError>>> {
        Box::pin(async move {
            let context = self.store.data_mut().resources.push(context.clone())?;

            let result = self
                .inner
                .grafbase_sdk_authorization()
                .call_authorize_response(&mut self.store, context, state, elements)
                .await?;

            Ok(result
                .map(Into::into)
                .map_err(|err| err.into_graphql_error(ErrorCode::Unauthorized)))
        })
    }
}
