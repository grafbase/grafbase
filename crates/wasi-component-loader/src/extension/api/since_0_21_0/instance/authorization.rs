use engine::{EngineOperationContext, EngineRequestContext};
use engine_error::{ErrorCode, ErrorResponse, GraphqlError};
use futures::future::BoxFuture;
use runtime::extension::AuthorizationDecisions;

use crate::extension::{
    AuthorizationExtensionInstance, AuthorizeQueryOutput,
    api::since_0_21_0::wit::{self, exports::grafbase::sdk::authorization::AuthorizationOutput},
};

impl AuthorizationExtensionInstance for super::ExtensionInstanceSince0_21_0 {
    fn authorize_query<'a>(
        &'a mut self,
        ctx: EngineRequestContext,
        headers: wit::Headers,
        elements: wit::QueryElements<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<AuthorizeQueryOutput, ErrorResponse>>> {
        Box::pin(async move {
            let resources = &mut self.store.data_mut().resources;
            let headers = resources.push(wit::Headers::from(headers))?;
            let host_context = resources.push(wit::HostContext::from(&ctx))?;
            let ctx = resources.push(ctx)?;

            let result = self
                .inner
                .grafbase_sdk_authorization()
                .call_authorize_query(&mut self.store, host_context, ctx, headers, elements)
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
        elements: wit::ResponseElements<'a>,
    ) -> BoxFuture<'a, wasmtime::Result<Result<AuthorizationDecisions, GraphqlError>>> {
        Box::pin(async move {
            let resources = &mut self.store.data_mut().resources;
            let host_context = resources.push(wit::HostContext::from(&ctx))?;
            let ctx = resources.push(ctx)?;

            let result = self
                .inner
                .grafbase_sdk_authorization()
                .call_authorize_response(&mut self.store, host_context, ctx, state, elements)
                .await?;

            Ok(result
                .map(Into::into)
                .map_err(|err| err.into_graphql_error(ErrorCode::Unauthorized)))
        })
    }
}
