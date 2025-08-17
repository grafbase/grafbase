use engine::{EngineOperationContext, EngineRequestContext};
use engine_error::{ErrorCode, ErrorResponse, GraphqlError};
use futures::future::BoxFuture;
use runtime::extension::AuthorizationDecisions;

use crate::{
    extension::{
        AuthorizationExtensionInstance, AuthorizeQueryOutput,
        api::{
            since_0_19_0::{wit::exports::grafbase::sdk::authorization::AuthorizationOutput, world as wit19},
            wit::{Headers, QueryElements, ResponseElements},
        },
    },
    resources::LegacyWasmContext,
};

impl AuthorizationExtensionInstance for super::ExtensionInstanceSince0_19_0 {
    fn authorize_query<'a>(
        &'a mut self,
        ctx: EngineRequestContext,
        headers: Headers,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<AuthorizeQueryOutput, ErrorResponse>>> {
        Box::pin(async move {
            let context = self.store.data_mut().resources.push(LegacyWasmContext::from(&ctx))?;
            let headers = self.store.data_mut().resources.push(headers)?;

            let token_param = ctx
                .token()
                .as_bytes()
                .map(wit19::TokenParam::Bytes)
                .unwrap_or(wit19::TokenParam::Anonymous);

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
                    Ok(AuthorizeQueryOutput {
                        subgraph_headers: headers,
                        additional_headers: None,
                        decisions: decisions.into(),
                        context: Default::default(),
                        state,
                    })
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
        ctx: EngineOperationContext,
        state: &'a [u8],
        elements: ResponseElements<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<AuthorizationDecisions, GraphqlError>>> {
        Box::pin(async move {
            let context = self.store.data_mut().resources.push(LegacyWasmContext::from(&ctx))?;

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
