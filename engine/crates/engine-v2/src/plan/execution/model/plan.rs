use schema::{EntityDefinition, ResolverDefinition};
use walker::Walk;

use crate::plan::{PlanId, PlanRecord};

use super::{QueryContext, SelectionSet};

#[derive(Clone, Copy)]
pub(crate) struct QueryPlan<'a> {
    pub(in crate::plan::execution::model) ctx: QueryContext<'a>,
    pub(in crate::plan::execution::model) id: PlanId,
}

#[allow(unused)]
impl<'a> QueryPlan<'a> {
    #[allow(clippy::should_implement_trait)]
    fn as_ref(&self) -> &'a PlanRecord {
        &self.ctx.operation_plan[self.id]
    }
    pub(crate) fn entity_definition(&self) -> EntityDefinition<'a> {
        self.as_ref().entity_definition_id.walk(self.ctx.schema)
    }
    pub(crate) fn resolver_definition(&self) -> ResolverDefinition<'a> {
        self.as_ref().resolver_definition_id.walk(self.ctx.schema)
    }
    pub(crate) fn selection_set(&self) -> SelectionSet<'a> {
        SelectionSet {
            ctx: self.ctx,
            item: self.as_ref().selection_set_record,
            requires_typename: false,
        }
    }
}

impl<'a> Walk<QueryContext<'a>> for PlanId {
    type Walker<'w> = QueryPlan<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: QueryContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        QueryPlan { ctx, id: self }
    }
}

impl std::fmt::Debug for QueryPlan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plan")
            .field("entity_definition", &self.entity_definition())
            .field("resolver_definition", &self.resolver_definition())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
