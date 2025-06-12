use anyhow::anyhow;
use futures::future::BoxFuture;
use runtime::extension::{AuthorizationDecisions, TokenRef};

use crate::{
    Error, SharedContext,
    extension::{
        AuthorizationExtensionInstance, QueryAuthorizationResult,
        api::wit::{QueryElements, ResponseElements},
    },
    resources::{AuthorizationContext, Lease},
};

impl AuthorizationExtensionInstance for super::ExtensionInstanceSince090 {
    fn authorize_query<'a>(
        &'a mut self,
        _: SharedContext,
        headers: Lease<http::HeaderMap>,
        token: TokenRef<'a>,
        elements: QueryElements<'a>,
    ) -> BoxFuture<'a, QueryAuthorizationResult> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;
            let ctx = AuthorizationContext {
                headers: headers.into(),
                token: token.to_owned(),
            };
            let ctx = self.store.data_mut().push_resource(ctx)?;
            let ctx_rep = ctx.rep();

            let result = self
                .inner
                .grafbase_sdk_authorization()
                .call_authorize_query(&mut self.store, ctx, elements)
                .await?;

            let AuthorizationContext { headers, .. } =
                self.store.data_mut().take_resource::<AuthorizationContext>(ctx_rep)?;

            self.poisoned = false;

            result
                .map(|(decisions, state)| (headers.into_lease().unwrap(), decisions.into(), state))
                .map_err(Into::into)
        })
    }

    fn authorize_response<'a>(
        &'a mut self,
        _: SharedContext,
        _: &'a [u8],
        _: ResponseElements<'a>,
    ) -> BoxFuture<'a, Result<AuthorizationDecisions, Error>> {
        Box::pin(async move {
            Err(Error::Internal(anyhow!(
                "SDK 0.9.0 had only experimental support for authorize_response."
            )))
        })
    }
}
