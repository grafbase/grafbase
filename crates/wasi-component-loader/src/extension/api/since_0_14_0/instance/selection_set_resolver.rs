use engine_error::{ErrorCode, GraphqlError};
use futures::future::BoxFuture;
use runtime::extension::Data;

use crate::{
    extension::{
        SelectionSetResolverExtensionInstance,
        api::wit::{ArgumentsId, Field, FieldId},
    },
    resources::LegacyHeaders,
};

impl SelectionSetResolverExtensionInstance for super::ExtensionInstanceSince0_14_0 {
    fn selection_set_resolver_prepare<'a>(
        &'a mut self,
        subgraph_name: &'a str,
        field_id: FieldId,
        fields: &'a [Field<'a>],
    ) -> BoxFuture<'a, wasmtime::Result<Result<Vec<u8>, GraphqlError>>> {
        Box::pin(async move {
            let result = self
                .inner
                .grafbase_sdk_selection_set_resolver()
                .call_prepare(&mut self.store, subgraph_name, field_id, fields)
                .await?;
            Ok(result.map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }

    fn resolve_query_or_mutation_field<'a>(
        &'a mut self,
        headers: http::HeaderMap,
        subgraph_name: &'a str,
        prepared: &'a [u8],
        arguments: &'a [(ArgumentsId, &'a [u8])],
    ) -> BoxFuture<'a, wasmtime::Result<Result<Data, GraphqlError>>> {
        Box::pin(async move {
            let headers = self.store.data_mut().resources.push(LegacyHeaders::from(headers))?;
            let result = self
                .inner
                .grafbase_sdk_selection_set_resolver()
                .call_resolve_query_or_mutation_field(&mut self.store, headers, subgraph_name, prepared, arguments)
                .await?;

            Ok(result
                .map(Into::into)
                .map_err(|err| err.into_graphql_error(ErrorCode::ExtensionError)))
        })
    }
}
