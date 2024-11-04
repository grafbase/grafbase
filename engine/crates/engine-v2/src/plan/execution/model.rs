use crate::{plan::PlanId, response::ResponseViewSelectionSet, subgraph::Resolver};

pub(crate) struct ExecutionPlan {
    pub plan_id: PlanId,
    pub requires: ResponseViewSelectionSet,
    pub resolver: Resolver,
}
