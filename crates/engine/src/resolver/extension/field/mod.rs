mod query_or_mutation;
mod subscription;

use futures::{FutureExt, TryStreamExt as _, stream::FuturesUnordered};
use runtime::extension::FieldResolverExtension as _;
use schema::{ExtensionDirectiveId, FieldResolverExtensionDefinition};

use crate::{
    Runtime,
    prepare::{PartitionDataFieldId, PlanQueryPartition, PlanResult, PrepareContext},
    resolver::Resolver,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct FieldResolverExtension {
    pub directive_id: ExtensionDirectiveId,
    prepared: Vec<PreparedField>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PreparedField {
    field_id: PartitionDataFieldId,
    extension_data: Vec<u8>,
}

impl FieldResolverExtension {
    pub(in crate::resolver) async fn prepare(
        ctx: &PrepareContext<'_, impl Runtime>,
        definition: FieldResolverExtensionDefinition<'_>,
        plan_query_partition: PlanQueryPartition<'_>,
    ) -> PlanResult<Resolver> {
        let prepared = plan_query_partition
            .selection_set()
            .fields()
            .map(|field| async move {
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
                PlanResult::Ok(PreparedField {
                    field_id: field.id,
                    extension_data: prepared_data,
                })
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        Ok(Resolver::FieldResolverExtension(Self {
            directive_id: definition.directive_id,
            prepared,
        }))
    }
}
