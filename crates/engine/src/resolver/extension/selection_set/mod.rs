mod query_or_mutation;

use futures::{FutureExt, TryStreamExt as _, stream::FuturesUnordered};
use id_newtypes::IdRange;
use runtime::extension::SelectionSetResolverExtension as _;
use schema::{SelectionSetResolverExtensionDefinition, SelectionSetResolverExtensionDefinitionRecord};

use crate::{
    Runtime,
    prepare::{DataOrLookupFieldId, PartitionFieldArgumentId, PlanResult, PrepareContext, SubgraphSelectionSet},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct SelectionSetResolverExtension {
    pub definition: SelectionSetResolverExtensionDefinitionRecord,
    prepared_fields: Vec<PreparedField>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PreparedField {
    id: DataOrLookupFieldId,
    extension_data: Vec<u8>,
    arguments: Vec<(runtime::extension::ArgumentsId, IdRange<PartitionFieldArgumentId>)>,
}

impl SelectionSetResolverExtension {
    pub(in crate::resolver) async fn prepare(
        ctx: &PrepareContext<'_, impl Runtime>,
        definition: SelectionSetResolverExtensionDefinition<'_>,
        selection_set: SubgraphSelectionSet<'_>,
    ) -> PlanResult<Self> {
        let prepared = selection_set
            .fields()
            .map(|field| async move {
                let prepared_data = ctx
                    .runtime()
                    .extensions()
                    .prepare(definition.extension_id, definition.subgraph().into(), field)
                    // FIXME: Unfortunately, boxing seems to be the only solution for the bug explained here:
                    //        https://github.com/rust-lang/rust/issues/110338#issuecomment-1513761297
                    .boxed()
                    .await?;

                let mut arguments = Vec::new();
                if let Some(id) = runtime::extension::Field::arguments(&field) {
                    arguments.push((id, field.argument_ids()))
                }
                let mut stack = vec![field.selection_set()];
                while let Some(selection_set) = stack.pop() {
                    for field in selection_set.fields() {
                        if let Some(id) = runtime::extension::Field::arguments(&field) {
                            arguments.push((id, field.argument_ids()))
                        }
                        let selection_set = field.selection_set();
                        if !selection_set.is_empty() {
                            stack.push(selection_set);
                        }
                    }
                }

                PlanResult::Ok(PreparedField {
                    id: field.id,
                    extension_data: prepared_data,
                    arguments,
                })
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        Ok(Self {
            definition: *definition,
            prepared_fields: prepared,
        })
    }
}
