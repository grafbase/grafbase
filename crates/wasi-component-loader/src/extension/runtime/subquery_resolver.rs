use engine_error::GraphqlError;
use engine_schema::FieldDefinition;
use runtime::{
    extension::{Data, SelectionSet, SubQueryResolverExtension},
    hooks::Anything,
};

use crate::{SharedContext, extension::WasmExtensions};

#[allow(clippy::manual_async_fn)]
impl SubQueryResolverExtension<SharedContext> for WasmExtensions {
    fn prepare<'ctx>(
        &'ctx self,
        _field_definition: FieldDefinition<'ctx>,
        _selection_set: impl SelectionSet<'ctx>,
    ) -> impl Future<Output = Result<Vec<u8>, GraphqlError>> + Send {
        async { Err(GraphqlError::internal_server_error()) }
    }

    fn resolve_query_or_mutation_field<'ctx, 'resp, 'f>(
        &'ctx self,
        _prepared_data: &'ctx [u8],
        _subgraph_headers: http::HeaderMap,
        _variables: impl Iterator<Item: Anything<'resp>> + Send,
    ) -> impl Future<Output = Result<Data, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        async { Err(GraphqlError::internal_server_error()) }
    }
}
