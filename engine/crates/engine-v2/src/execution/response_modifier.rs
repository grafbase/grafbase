use std::sync::Arc;

use itertools::Itertools;

use crate::{
    operation::ResponseModifierRule,
    response::{ErrorCode, GraphqlError, InputdResponseObjectSet, ResponseBuilder, UnpackedResponseEdge},
    Runtime,
};

use super::{state::OperationExecutionState, ExecutionContext, ResponseModifierExecutorId};

impl<'ctx, R: Runtime> ExecutionContext<'ctx, R> {
    pub(super) async fn execute_response_modifier(
        &self,
        state: &mut OperationExecutionState<'ctx>,
        response: &mut ResponseBuilder,
        response_modifier_executor_id: ResponseModifierExecutorId,
    ) {
        let executor = &self.operation[response_modifier_executor_id];
        // First step is aggregating all the object refs we must read into a single
        // InputdResponseObjectSet.
        // As the AuthorizedField resolver applies on a specific field, we have to keep track of
        // which ResponseKeys (~field in the output) would be impacted if unauthorized. As multiple
        // fields might be impacted in a given object set (because of aliases), we keep a range of
        // of those keys for each ResponseObjectSet we add to the input.
        let mut input = InputdResponseObjectSet::default();
        let mut input_associated_key_range = Vec::new();
        for (set_id, chunk) in executor
            .on
            .iter()
            .enumerate()
            .chunk_by(|(_, (set_id, _, _))| *set_id)
            .into_iter()
        {
            let refs = state[set_id]
                .as_ref()
                .expect("Response Modifier is ready but response object set doesn't exist");

            for (entity_id, mut chunk) in chunk.into_iter().chunk_by(|(_, (_, entity, _))| *entity).into_iter() {
                let start = chunk.next().unwrap().0;
                let end = chunk.last().map(|(ix, _)| ix).unwrap_or(start) + 1;
                input_associated_key_range.push(start..end);

                if let Some(entity_id) = entity_id {
                    input = input.with_filtered_response_objects(self.schema(), entity_id, refs.clone());
                } else {
                    input = input.with_response_objects(refs.clone());
                }
            }
        }

        // Now we can execute the hook and propagate any errors.
        match executor.rule {
            ResponseModifierRule::AuthorizedField {
                directive_id,
                definition_id,
            } => {
                let definition = self.schema().walk(definition_id);
                let directive = self.schema().walk(directive_id);
                let input = Arc::new(input);
                let parents = response.read(
                    self.schema(),
                    &self.operation.response_views,
                    input.clone(),
                    executor.requires,
                );
                let result = self
                    .hooks()
                    .authorize_parent_edge_post_execution(definition, parents, directive.metadata())
                    .await;
                // FIXME: make this efficient
                let result = match result {
                    Ok(result) => {
                        if result.len() == input.len() {
                            result
                                .into_iter()
                                .map(|res| res.map_err(GraphqlError::from))
                                .collect::<Vec<_>>()
                        } else {
                            // TODO: should be an error log instead not add any GraphQL error I
                            // guess
                            (0..input.len())
                                .map(|_| {
                                    Err(GraphqlError::new(
                                        "Incorrect number of authorization replies",
                                        ErrorCode::HookError,
                                    ))
                                })
                                .collect()
                        }
                    }
                    Err(err) => (0..input.len()).map(|_| Err(err.clone())).collect(),
                };

                for ((i, obj_ref), result) in input.iter_with_set_index().zip_eq(result) {
                    if let Err(err) = result {
                        for (_, _, key) in &executor.on[input_associated_key_range[i].clone()] {
                            let path = obj_ref
                                .path
                                .child(UnpackedResponseEdge::ExtraFieldResponseKey(*key).pack());
                            response.push_error(err.clone().with_path(path));
                        }
                    }
                }
            }
        }
    }
}
