use engine::{EngineOperationContext, EngineRequestContext};
use engine_error::{ErrorCode, ErrorResponse, GraphqlError};
use futures::future::BoxFuture;
use runtime::extension::{AuthorizationDecisions, TokenRef};

use crate::{
    extension::{
        AuthorizationExtensionInstance, AuthorizeQueryOutput,
        api::{
            since_0_15_0::world as wit15,
            wit::{QueryElements, ResponseElements},
        },
    },
    resources::{LegacyHeaders, OwnedOrShared},
};

impl AuthorizationExtensionInstance for super::ExtensionInstanceSince0_15_0 {
    fn authorize_query<'a>(
        &'a mut self,
        ctx: EngineRequestContext,
        headers: OwnedOrShared<http::HeaderMap>,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<AuthorizeQueryOutput, ErrorResponse>>> {
        Box::pin(async move {
            let headers = self.store.data_mut().resources.push(LegacyHeaders::from(headers))?;
            let headers_rep = headers.rep();

            let token_param = ctx
                .token()
                .as_bytes()
                .map(wit15::TokenParam::Bytes)
                .unwrap_or(wit15::TokenParam::Anonymous);
            let result = self
                .inner
                .grafbase_sdk_authorization()
                .call_authorize_query(&mut self.store, headers, token_param, elements.into())
                .await?;

            let headers = self.store.data_mut().take_leased_resource(headers_rep)?;

            let result = match result {
                Ok((decisions, state)) => Ok(AuthorizeQueryOutput {
                    subgraph_headers: headers,
                    additional_headers: None,
                    decisions: decisions.into(),
                    context: Default::default(),
                    state,
                }),
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
        _ctx: EngineOperationContext,
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
