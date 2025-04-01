use engine_error::GraphqlError;
use engine_schema::{FieldDefinition, Subgraph};
use extension_catalog::ExtensionId;
use runtime::{
    extension::{ArgumentsId, Data, SelectionSet, SelectionSetResolverExtension},
    hooks::Anything,
};

use crate::{SharedContext, extension::WasmExtensions};

#[allow(clippy::manual_async_fn)]
impl SelectionSetResolverExtension<SharedContext> for WasmExtensions {
    fn prepare<'ctx>(
        &'ctx self,
        _extension_id: ExtensionId,
        _subgraph: Subgraph<'ctx>,
        _field_definition: FieldDefinition<'ctx>,
        _selection_set: impl SelectionSet<'ctx>,
    ) -> impl Future<Output = Result<Vec<u8>, GraphqlError>> + Send {
        async { Err(GraphqlError::internal_server_error()) }
    }

    fn resolve_query_or_mutation_field<'ctx, 'resp, 'f>(
        &'ctx self,
        _extension_id: ExtensionId,
        _subgraph: Subgraph<'ctx>,
        _prepared_data: &'ctx [u8],
        _subgraph_headers: http::HeaderMap,
        _arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = Result<Data, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        async { Err(GraphqlError::internal_server_error()) }
    }
}
