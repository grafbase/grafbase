use engine::{EngineOperationContext, EngineRequestContext};
use engine_error::{ErrorCode, ErrorResponse, GraphqlError};
use futures::future::BoxFuture;
use runtime::extension::{AuthorizationDecisions, TokenRef};

use crate::extension::{
    AuthorizationExtensionInstance, AuthorizeQueryOutput,
    api::{
        since_0_21_0::wit::exports::grafbase::sdk::authorization::AuthorizationOutput,
        wit::{Headers, QueryElements, ResponseElements},
    },
};

impl AuthorizationExtensionInstance for super::ExtensionInstanceSince0_21_0 {
    fn authorize_query<'a>(
        &'a mut self,
        ctx: EngineRequestContext,
        headers: Headers,
        token: TokenRef<'a>,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<AuthorizeQueryOutput, ErrorResponse>>> {
        Box::pin(async move {
            let context = self.store.data_mut().resources.push(context.clone())?;
            let headers = self.store.data_mut().resources.push(headers)?;

            let result = self
                .inner
                .grafbase_sdk_authorization()
                .call_authorize_query(&mut self.store, context, headers, elements)
                .await?;

            let result = match result {
                Ok(AuthorizationOutput {
                    decisions,
                    context,
                    state,
                    subgraph_headers,
                    additional_headers,
                }) => {
                    let resources = &mut self.store.data_mut().resources;
                    let subgraph_headers = resources.delete(subgraph_headers)?;
                    let additional_headers = additional_headers
                        .map(|headers| resources.delete(headers))
                        .transpose()?
                        .map(|headers| headers.into_inner().unwrap());
                    Ok(AuthorizeQueryOutput {
                        subgraph_headers,
                        additional_headers,
                        decisions: decisions.into(),
                        context,
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
