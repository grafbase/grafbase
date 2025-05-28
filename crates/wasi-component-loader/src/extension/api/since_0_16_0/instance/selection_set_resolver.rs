use engine_error::{ErrorCode, GraphqlError};
use futures::future::BoxFuture;
use runtime::extension::Data;

use crate::{
    Error,
    extension::{
        SelectionSetResolverExtensionInstance,
        api::wit::{ArgumentsId, Field, FieldId},
    },
    resources::Headers,
};

impl SelectionSetResolverExtensionInstance for super::ExtensionInstanceSince0_16_0 {
    fn prepare<'a>(
        &'a mut self,
        subgraph_name: &'a str,
        field_id: FieldId,
        fields: &'a [Field<'a>],
    ) -> BoxFuture<'a, Result<Result<Vec<u8>, GraphqlError>, Error>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;
            let result = self
                .inner
                .grafbase_sdk_selection_set_resolver()
                .call_prepare(&mut self.store, subgraph_name, field_id, fields)
                .await?;

            self.poisoned = false;
            Ok(result.map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }

    fn resolve_query_or_mutation_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        prepared: &'a [u8],
        arguments: &'a [(ArgumentsId, &'a [u8])],
    ) -> BoxFuture<'a, Result<Result<Data, GraphqlError>, Error>> {
        Box::pin(async move {
            // Futures may be canceled, so we pro-actively mark the instance as poisoned until proven
            // otherwise.
            self.poisoned = true;
            let headers = self.store.data_mut().push_resource(Headers::from(headers))?;
            let result = self
                .inner
                .grafbase_sdk_selection_set_resolver()
                .call_resolve_query_or_mutation_field(&mut self.store, headers, subgraph_name, prepared, arguments)
                .await?;
            self.poisoned = false;
            Ok(result
                .map(Into::into)
                .map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }
}
