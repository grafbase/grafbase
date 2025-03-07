mod query_or_mutation;
mod subscription;

use schema::{ExtensionDirectiveId, FieldResolverExtensionDefinition};

use crate::prepare::{PartitionDataFieldId, PlanQueryPartition};

use super::Resolver;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct FieldResolverExtension {
    pub directive_id: ExtensionDirectiveId,
    pub field_id: PartitionDataFieldId,
}

impl FieldResolverExtension {
    pub(in crate::resolver) fn prepare(
        definition: FieldResolverExtensionDefinition<'_>,
        plan_query_partition: PlanQueryPartition<'_>,
    ) -> Resolver {
        let field = plan_query_partition
            .selection_set()
            .fields()
            .next()
            .expect("At least one field must be present");

        Resolver::FieldResolverExtension(Self {
            directive_id: definition.directive_id,
            field_id: field.id,
        })
    }
}
