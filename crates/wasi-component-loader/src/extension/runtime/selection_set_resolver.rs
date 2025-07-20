use engine_error::GraphqlError;
use engine_schema::Subgraph;
use extension_catalog::ExtensionId;
use runtime::extension::{Anything, ArgumentsId, Data, Field as _, SelectionSet as _, SelectionSetResolverExtension};

use crate::{
    cbor,
    extension::{
        EngineWasmExtensions,
        api::wit::{self, Field, SelectionSet},
    },
    wasmsafe,
};

#[allow(clippy::manual_async_fn)]
impl SelectionSetResolverExtension for EngineWasmExtensions {
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

        wasmsafe!(
            instance
                .selection_set_resolver_prepare(subgraph.name(), 0, &fields)
                .await
        )
    }

    fn resolve<'ctx, 'resp, 'f>(
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
            wasmsafe!(
                instance
                    .resolve_query_or_mutation_field(subgraph_headers, subgraph.name(), prepared_data, &arguments_refs)
                    .await
            )
        }
    }
}
