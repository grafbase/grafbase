use futures::future::BoxFuture;
use runtime::extension::{AuthorizationDecisions, TokenRef};

use crate::{
    Error, ErrorResponse, SharedContext,
    extension::{
        AuthorizationExtensionInstance, QueryAuthorizationResult,
        api::wit::{Headers, QueryElements, ResponseElements, TokenParam},
    },
    resources::Lease,
};

impl AuthorizationExtensionInstance for super::ExtensionInstanceSince0_19_0 {
    fn authorize_query<'a>(
        &'a mut self,
        context: SharedContext,
        headers: Lease<http::HeaderMap>,
        token: TokenRef<'a>,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, QueryAuthorizationResult> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let context = self.store.data_mut().push_resource(context)?;
            let headers = self.store.data_mut().push_resource(Headers::from(headers))?;
            let headers_rep = headers.rep();

            let token_param = token.as_bytes().map(TokenParam::Bytes).unwrap_or(TokenParam::Anonymous);

            let result = self
                .inner
                .grafbase_sdk_authorization()
                .call_authorize_query(&mut self.store, context, headers, token_param, elements)
                .await?;

            let headers = self
                .store
                .data_mut()
                .take_resource::<Headers>(headers_rep)?
                .into_lease()
                .unwrap();

            self.poisoned = false;

            result
                .map(|(decisions, state)| (headers, decisions.into(), state))
                .map_err(|err| ErrorResponse::from_wit(&mut self.store, err))
        })
    }

    fn authorize_response<'a>(
        &'a mut self,
        context: SharedContext,
        state: &'a [u8],
        elements: ResponseElements<'a>,
    ) -> BoxFuture<'a, Result<AuthorizationDecisions, Error>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let context = self.store.data_mut().push_resource(context)?;

            let result = self
                .inner
                .grafbase_sdk_authorization()
                .call_authorize_response(&mut self.store, context, state, elements)
                .await?;

            self.poisoned = false;
            result.map(Into::into).map_err(Into::into)
        })
    }
}
