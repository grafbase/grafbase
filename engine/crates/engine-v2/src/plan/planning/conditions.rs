use crate::{
    operation::{Condition, ConditionResult},
    response::{ErrorCode, GraphqlError},
    Runtime,
};

use super::{collect::OperationPlanBuilder, PlanningResult};

impl<'ctx, 'op, R: Runtime> OperationPlanBuilder<'ctx, 'op, R>
where
    'ctx: 'op,
{
    pub(super) async fn evaluate_all_conditions(&self) -> PlanningResult<Vec<ConditionResult>> {
        let mut results = Vec::with_capacity(self.operation_plan.conditions.len());

        let is_anonymous = self.ctx.access_token().is_anonymous();
        let mut scopes = None;

        for condition in &self.operation_plan.conditions {
            let result = match condition {
                Condition::All(ids) => ids
                    .iter()
                    .map(|id| &results[usize::from(*id)])
                    .fold(ConditionResult::Include, |current, cond| current & cond),
                Condition::Authenticated => {
                    if is_anonymous {
                        ConditionResult::Errors(vec![GraphqlError::new("Unauthenticated", ErrorCode::Unauthenticated)])
                    } else {
                        ConditionResult::Include
                    }
                }
                Condition::RequiresScopes(id) => {
                    let scopes = scopes.get_or_insert_with(|| {
                        self.ctx
                            .access_token()
                            .get_claim("scope")
                            .as_str()
                            .map(|scope| scope.split(' ').collect::<Vec<_>>())
                            .unwrap_or_default()
                    });

                    if self.ctx.schema.walk(*id).matches(scopes) {
                        ConditionResult::Include
                    } else {
                        ConditionResult::Errors(vec![GraphqlError::new(
                            "Not allowed: insufficient scopes",
                            ErrorCode::Unauthorized,
                        )])
                    }
                }
                Condition::AuthorizedEdge { directive_id, field_id } => {
                    let directive = &self.ctx.schema[*directive_id];
                    let field = self.walker().walk(*field_id);
                    let arguments = field.arguments().with_selection_set(&directive.arguments);

                    let result = self
                        .ctx
                        .hooks()
                        .authorize_edge_pre_execution(
                            field.definition().expect("@authorized cannot be applied on __typename"),
                            arguments,
                            directive.metadata.map(|id| self.ctx.schema.walk(&self.ctx.schema[id])),
                        )
                        .await;
                    if let Err(err) = result {
                        ConditionResult::Errors(vec![err])
                    } else {
                        ConditionResult::Include
                    }
                }
                Condition::AuthorizedNode {
                    directive_id,
                    entity_id,
                } => {
                    let directive = &self.ctx.schema[*directive_id];
                    let result = self
                        .ctx
                        .hooks()
                        .authorize_node_pre_execution(
                            self.ctx.schema.walk(*entity_id),
                            directive.metadata.map(|id| self.ctx.schema.walk(&self.ctx.schema[id])),
                        )
                        .await;

                    if let Err(err) = result {
                        ConditionResult::Errors(vec![err])
                    } else {
                        ConditionResult::Include
                    }
                }
            };
            results.push(result);
        }

        Ok(results)
    }
}
