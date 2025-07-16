mod query_or_mutation;
mod subscription;

use futures::{TryStreamExt as _, stream::FuturesUnordered};
use schema::{ExtensionDirectiveId, FieldResolverExtensionDefinition};

use crate::{
    Runtime,
    prepare::{DataOrLookupFieldId, PlanResult, PrepareContext, SubgraphSelectionSet},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct FieldResolverExtension {
    pub directive_id: ExtensionDirectiveId,
    prepared: Vec<PreparedField>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PreparedField {
    field_id: DataOrLookupFieldId,
}

impl FieldResolverExtension {
    pub(in crate::resolver) async fn prepare(
        _ctx: &PrepareContext<'_, impl Runtime>,
        definition: FieldResolverExtensionDefinition<'_>,
        selection_set: SubgraphSelectionSet<'_>,
    ) -> PlanResult<Self> {
        let prepared = selection_set
            .fields()
            .map(|field| async move { PlanResult::Ok(PreparedField { field_id: field.id }) })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        Ok(Self {
            directive_id: definition.directive_id,
            prepared,
        })
    }
}
