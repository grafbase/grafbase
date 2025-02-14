use schema::{EntityDefinition, ResolverDefinition};
use walker::Walk;

use crate::prepare::{QueryPartitionId, QueryPartitionRecord};

use super::{OperationPlanContext, SubgraphSelectionSet};

#[derive(Clone, Copy)]
pub(crate) struct PlanQueryPartition<'a> {
    pub(in crate::prepare::operation_plan::model) ctx: OperationPlanContext<'a>,
    pub(in crate::prepare::operation_plan::model) id: QueryPartitionId,
}

#[allow(unused)]
impl<'a> PlanQueryPartition<'a> {
    // Not providing too easy access to the query partition as it exposes the unfiltered fields
    // before query modifications. It's likely not what you want.
    fn as_ref(&self) -> &'a QueryPartitionRecord {
        &self.ctx.cached.query_plan[self.id]
    }

    pub(crate) fn entity_definition(&self) -> EntityDefinition<'a> {
        self.as_ref().entity_definition_id.walk(self.ctx)
    }

    pub(crate) fn resolver_definition(&self) -> ResolverDefinition<'a> {
        self.as_ref().resolver_definition_id.walk(self.ctx)
    }

    pub(crate) fn selection_set(&self) -> SubgraphSelectionSet<'a> {
        SubgraphSelectionSet {
            ctx: self.ctx,
            item: self.as_ref().selection_set_record,
            // If we may encounter an inaccessible object, we have to detect it
            requires_typename: self
                .entity_definition()
                .as_interface()
                .map(|inf| inf.has_inaccessible_implementor())
                .unwrap_or_default(),
        }
    }
}

impl std::fmt::Debug for PlanQueryPartition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanQueryPartition").finish()
    }
}

impl<'a> OperationPlanContext<'a> {
    pub(crate) fn view(&self, id: QueryPartitionId) -> PlanQueryPartition<'a> {
        PlanQueryPartition { ctx: *self, id }
    }
}
