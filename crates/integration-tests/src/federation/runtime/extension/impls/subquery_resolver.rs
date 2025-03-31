use engine::GraphqlError;
use engine_schema::FieldDefinition;
use runtime::{
    extension::{Data, SelectionSet, SubQueryResolverExtension},
    hooks::Anything,
};

use crate::federation::{DynHookContext, ExtContext, ExtensionsDispatcher, TestExtensions};

#[allow(clippy::manual_async_fn, unused_variables)]
impl SubQueryResolverExtension<ExtContext> for ExtensionsDispatcher {
    fn prepare<'ctx>(
        &'ctx self,
        field_definition: FieldDefinition<'ctx>,
        selection_set: impl SelectionSet<'ctx>,
    ) -> impl Future<Output = Result<Vec<u8>, GraphqlError>> + Send {
        async { Err(GraphqlError::internal_server_error()) }
    }

    fn resolve_query_or_mutation_field<'ctx, 'resp, 'f>(
        &'ctx self,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        variables: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Data, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        async { Err(GraphqlError::internal_server_error()) }
    }
}

#[allow(clippy::manual_async_fn, unused_variables)]
impl SubQueryResolverExtension<DynHookContext> for TestExtensions {
    fn prepare<'ctx>(
        &'ctx self,
        field_definition: FieldDefinition<'ctx>,
        selection_set: impl runtime::extension::SelectionSet<'ctx>,
    ) -> impl Future<Output = Result<Vec<u8>, GraphqlError>> + Send {
        async { Err(GraphqlError::internal_extension_error()) }
    }

    fn resolve_query_or_mutation_field<'ctx, 'resp, 'f>(
        &'ctx self,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        variables: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Data, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        async { Err(GraphqlError::internal_extension_error()) }
    }
}
