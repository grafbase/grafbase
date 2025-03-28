mod query_or_mutation;
mod subscription;

use futures::FutureExt;
use runtime::extension::FieldResolverExtension as _;
use schema::{ExtensionDirectiveId, FieldResolverExtensionDefinition};

use crate::{
    Runtime,
    prepare::{PartitionDataFieldId, PlanQueryPartition, PlanResult, PrepareContext},
};

use super::Resolver;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct FieldResolverExtension {
    pub directive_id: ExtensionDirectiveId,
    pub field_id: PartitionDataFieldId,
    pub prepared_data: Vec<u8>,
}

impl FieldResolverExtension {
    pub(in crate::resolver) async fn prepare(
        ctx: &PrepareContext<'_, impl Runtime>,
        definition: FieldResolverExtensionDefinition<'_>,
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
            .prepare(
                definition.directive(),
                field.definition(),
                definition.directive().static_arguments(),
            )
            // FIXME: Unfortunately, boxing seems to be the only solution for the bug explained here:
            //        https://github.com/rust-lang/rust/issues/110338#issuecomment-1513761297
            .boxed()
            .await?;

        Ok(Resolver::FieldResolverExtension(Self {
            directive_id: definition.directive_id,
            field_id: field.id,
            prepared_data,
        }))
    }
}
