mod query_or_mutation;

use futures::FutureExt;
use query_solver::QueryOrSchemaFieldArgumentIds;
use runtime::extension::SelectionSetResolverExtension as _;
use schema::{SelectionSetResolverExtensionDefinition, SelectionSetResolverExtensionDefinitionRecord};

use crate::{
    Runtime,
    prepare::{PartitionDataFieldId, PlanQueryPartition, PlanResult, PrepareContext},
    resolver::Resolver,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct SelectionSetResolverExtension {
    pub definition: SelectionSetResolverExtensionDefinitionRecord,
    pub field_id: PartitionDataFieldId,
    pub prepared_data: Vec<u8>,
    pub arguments: Vec<(runtime::extension::ArgumentsId, QueryOrSchemaFieldArgumentIds)>,
}

impl SelectionSetResolverExtension {
    pub(in crate::resolver) async fn prepare(
        ctx: &PrepareContext<'_, impl Runtime>,
        definition: SelectionSetResolverExtensionDefinition<'_>,
        plan_query_partition: PlanQueryPartition<'_>,
    ) -> PlanResult<Resolver> {
        let field = plan_query_partition
            .selection_set()
            .fields()
            .next()
            .expect("At least one field must be present");

        let prepared_data = ctx
            .runtime()
            .extensions()
            .prepare(definition.extension_id, definition.subgraph().into(), field)
            // FIXME: Unfortunately, boxing seems to be the only solution for the bug explained here:
            //        https://github.com/rust-lang/rust/issues/110338#issuecomment-1513761297
            .boxed()
            .await?;

        let mut arguments = Vec::new();
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

        Ok(Resolver::SelectionSetResolverExtension(Self {
            definition: *definition,
            field_id: field.id,
            prepared_data,
            arguments,
        }))
    }
}
