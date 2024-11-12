use std::sync::Arc;

use itertools::Itertools;
use walker::Walk;

use crate::{
    operation::ResponseModifierRule,
    plan::ResponseModifier,
    response::{ErrorCode, GraphqlError, InputResponseObjectSet, ResponseBuilder, UnpackedResponseEdge},
    Runtime,
};

use super::{state::OperationExecutionState, ExecutionContext};

impl<'ctx, R: Runtime> ExecutionContext<'ctx, R> {
    pub(super) async fn execute_response_modifier(
        &self,
        state: &mut OperationExecutionState<'ctx, R>,
        response: &mut ResponseBuilder,
        response_modifier: ResponseModifier<'ctx>,
    ) {
        // First step is aggregating all the object refs we must read into a single
        // InputdResponseObjectSet.
        // As the AuthorizedField resolver applies on a specific field, we have to keep track of
        // which ResponseKeys (~field in the output) would be impacted if unauthorized. As multiple
        // fields might be impacted in a given object set (because of aliases), we keep a range of
        // of those keys for each ResponseObjectSet we add to the input.
        let mut input = InputResponseObjectSet::default();
        let mut input_associated_targets_range = Vec::new();
        for (set_id, chunk) in response_modifier
            .sorted_targets()
            .enumerate()
            .chunk_by(|(_, target)| target.set_id)
            .into_iter()
        {
            // With query modifications, this response object set might never exist.
            let Some(refs) = state[set_id].as_ref() else {
                continue;
            };

            for (ty_id, mut chunk) in chunk.into_iter().chunk_by(|(_, target)| target.ty_id).into_iter() {
                let (start, _) = chunk.next().unwrap();
                let end = chunk.last().map(|(last, _)| last).unwrap_or(start) + 1;
                input_associated_targets_range.push(start..end);

                if self.operation.solution[set_id].ty_id == ty_id {
                    input = input.with_response_objects(refs.clone());
                } else {
                    input = input.with_filtered_response_objects(self.schema(), ty_id, refs.clone());
                }
            }
        }

        if input.is_empty() {
            return;
        }

        // Now we can execute the hook and propagate any errors.
        match response_modifier.definition().rule {
            ResponseModifierRule::AuthorizedParentEdge {
                directive_id,
                definition_id,
            } => {
                let definition = self.schema().walk(definition_id);
                let directive = self.schema().walk(directive_id);
                let input = Arc::new(input);
                let parents = response.read(
                    self.schema(),
                    input.clone(),
                    directive_id.walk(self).fields().unwrap().as_ref(),
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
                        for target in
                            &response_modifier.sorted_target_records[input_associated_targets_range[i].clone()]
                        {
                            let path = obj_ref
                                .path
                                .child(UnpackedResponseEdge::ExtraFieldResponseKey(target.key.into()).pack());
                            response.push_error(err.clone().with_path(path));
                        }
                    }
                }
            }
            ResponseModifierRule::AuthorizedEdgeChild {
                directive_id,
                definition_id,
            } => {
                let definition = self.schema().walk(definition_id);
                let directive = self.schema().walk(directive_id);
                let input = Arc::new(input);
                let nodes = response.read(
                    self.schema(),
                    input.clone(),
                    directive_id.walk(self).node().unwrap().as_ref(),
                );
                let result = self
                    .hooks()
                    .authorize_edge_node_post_execution(definition, nodes, directive.metadata())
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

                for (obj_ref, result) in input.iter().zip_eq(result) {
                    if let Err(err) = result {
                        response.push_error(err.clone().with_path(obj_ref.path.clone()));
                    }
                }
            }
        }
    }
}
