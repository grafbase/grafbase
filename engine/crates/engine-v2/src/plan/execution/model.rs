use crate::{plan::PlanId, resolver::Resolver, response::ResponseViewSelectionSet};

pub(crate) struct ExecutionPlan {
    pub plan_id: PlanId,
    pub requires: ResponseViewSelectionSet,
    pub resolver: Resolver,
}
