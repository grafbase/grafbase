use engine_error::{ErrorCode, GraphqlError};
use futures::future::BoxFuture;

use crate::{
    Error, SharedContext,
    extension::{
        ResolverExtensionInstance,
        api::wit::{ArgumentsId, Directive, Field, FieldId, Response, SubscriptionItem},
    },
    resources::Headers,
};

impl ResolverExtensionInstance for super::ExtensionInstanceSince0_17_0 {
    fn prepare<'a>(
        &'a mut self,
        context: SharedContext,
        subgraph_name: &'a str,
        directive: Directive<'a>,
        field_id: FieldId,
        fields: &'a [Field<'a>],
    ) -> BoxFuture<'a, Result<Result<Vec<u8>, GraphqlError>, Error>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let context = self.store.data_mut().push_resource(context)?;

            let result = self
                .inner
                .grafbase_sdk_resolver()
                .call_prepare(&mut self.store, context, subgraph_name, directive, field_id, fields)
                .await?;

            self.poisoned = false;
            Ok(result.map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }

    fn resolve<'a>(
        &'a mut self,
        context: SharedContext,
        headers: http::HeaderMap,
        prepared: &'a [u8],
        arguments: &'a [(ArgumentsId, &'a [u8])],
    ) -> BoxFuture<'a, Result<Response, Error>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let headers = self.store.data_mut().push_resource(Headers::from(headers))?;
            let context = self.store.data_mut().push_resource(context)?;

            let response = self
                .inner
                .grafbase_sdk_resolver()
                .call_resolve(&mut self.store, context, prepared, headers, arguments)
                .await?;

            self.poisoned = false;

            Ok(response.into())
        })
    }

    fn create_subscription<'a>(
        &'a mut self,
        context: SharedContext,
        headers: http::HeaderMap,
        prepared: &'a [u8],
        arguments: &'a [(ArgumentsId, &'a [u8])],
    ) -> BoxFuture<'a, Result<Result<Option<Vec<u8>>, GraphqlError>, Error>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;

            let headers = self.store.data_mut().push_resource(Headers::from(headers))?;
            let context = self.store.data_mut().push_resource(context)?;

            let result = self
                .inner
                .grafbase_sdk_resolver()
                .call_create_subscription(&mut self.store, context, prepared, headers, arguments)
                .await?;

            // We don't remove poison flag here, as the subscription will be dropped later.
            Ok(result.map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }

    fn drop_subscription<'a>(&'a mut self, context: SharedContext) -> BoxFuture<'a, Result<(), Error>> {
        // We don't need to poison here, as it's already poisoned by create_subscription
        Box::pin(async move {
            let context = self.store.data_mut().push_resource(context)?;

            self.inner
                .grafbase_sdk_resolver()
                .call_drop_subscription(&mut self.store, context)
                .await?;

            self.poisoned = false;
            Ok(())
        })
    }

    fn resolve_next_subscription_item(
        &mut self,
        context: SharedContext,
    ) -> BoxFuture<'_, Result<Result<Option<SubscriptionItem>, GraphqlError>, Error>> {
        // We don't need to poison here, as it's already poisoned until we drop the subscription.
        Box::pin(async move {
            let context = self.store.data_mut().push_resource(context)?;

            let result = self
                .inner
                .grafbase_sdk_resolver()
                .call_resolve_next_subscription_item(&mut self.store, context)
                .await?;

            Ok(result
                .map(|i| i.map(Into::into))
                .map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }
}
