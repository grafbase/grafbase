use engine_error::{ErrorCode, GraphqlError};
use engine_schema::Subgraph;
use extension_catalog::ExtensionId;
use runtime::{
    extension::{ArgumentsId, Data, Field as _, SelectionSet as _, SelectionSetResolverExtension},
    hooks::Anything,
};

use crate::{
    Error, SharedContext, cbor,
    extension::{
        WasmExtensions,
        api::wit::{self, Field, SelectionSet},
    },
};

#[allow(clippy::manual_async_fn)]
impl SelectionSetResolverExtension<SharedContext> for WasmExtensions {
    async fn prepare<'ctx, F: runtime::extension::Field<'ctx>>(
        &'ctx self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'ctx>,
        field: F,
    ) -> Result<Vec<u8>, GraphqlError> {
        let mut instance = self.get(extension_id).await?;
        let mut fields = Vec::new();

        fields.push(Field {
            alias: field.alias(),
            definition_id: field.definition().id.as_guid(),
            arguments: field.arguments().map(Into::into),
            selection_set: None,
        });

        if let Some(selection_set) = field.selection_set() {
            let mut stack: Vec<(usize, F::SelectionSet)> = vec![(0, selection_set)];

            while let Some((field_id, selection_set)) = stack.pop() {
                let start = fields.len();
                for field in selection_set.fields_ordered_by_parent_entity() {
                    fields.push(Field {
                        alias: field.alias(),
                        definition_id: field.definition().id.as_guid(),
                        arguments: field.arguments().map(Into::into),
                        selection_set: None,
                    });
                    if let Some(selection_set) = field.selection_set() {
                        stack.push((fields.len() - 1, selection_set));
                    }
                }
                fields[field_id].selection_set = Some(SelectionSet {
                    requires_typename: selection_set.requires_typename(),
                    fields_ordered_by_parent_entity: (start as u16, fields.len() as u16),
                });
            }
        }

        instance
            .prepare(subgraph.name(), 0, &fields)
            .await
            .map_err(|err| match err {
                Error::Internal(err) => {
                    tracing::error!("Wasm error: {err}");
                    GraphqlError::new("Internal error", ErrorCode::ExtensionError)
                }
                Error::Guest(err) => err.into_graphql_error(ErrorCode::ExtensionError),
            })?
    }

    fn resolve_query_or_mutation_field<'ctx, 'resp, 'f>(
        &'ctx self,
        extension_id: ExtensionId,
        subgraph: Subgraph<'ctx>,
        prepared_data: &'ctx [u8],
        subgraph_headers: http::HeaderMap,
        arguments: impl Iterator<Item = (ArgumentsId, impl Anything<'resp>)> + Send,
    ) -> impl Future<Output = Result<Data, GraphqlError>> + Send + 'f
    where
        'ctx: 'f,
    {
        let arguments = arguments
            .map(|(id, value)| (id.into(), cbor::to_vec(&value).unwrap()))
            .collect::<Vec<(wit::ArgumentsId, Vec<u8>)>>();
        async move {
            let arguments_refs = arguments
                .iter()
                .map(|(id, value)| (*id, value.as_slice()))
                .collect::<Vec<_>>();
            let mut instance = self.get(extension_id).await?;
            instance
                .resolve_query_or_mutation_field(subgraph_headers, subgraph.name(), prepared_data, &arguments_refs)
                .await
                .map_err(|err| match err {
                    Error::Internal(err) => {
                        tracing::error!("Wasm error: {err}");
                        GraphqlError::new("Internal error", ErrorCode::ExtensionError)
                    }
                    Error::Guest(err) => err.into_graphql_error(ErrorCode::ExtensionError),
                })?
        }
    }
}
