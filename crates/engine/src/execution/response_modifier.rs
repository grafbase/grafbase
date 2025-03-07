use std::sync::Arc;

use itertools::Itertools;
use walker::Walk;

use crate::{
    Runtime,
    prepare::{ResponseModifier, ResponseModifierRule},
    response::{ErrorCode, GraphqlError, InputResponseObjectSet, ResponseBuilder, ResponseValueId},
};

use super::{ExecutionContext, state::OperationExecutionState};

impl<'ctx, R: Runtime> ExecutionContext<'ctx, R> {
    pub(super) async fn execute_response_modifier(
        &self,
        state: &mut OperationExecutionState<'ctx, R>,
        response: &mut ResponseBuilder,
        response_modifier: ResponseModifier<'ctx>,
    ) {
        for target in response_modifier.sorted_targets() {
            let Some(refs) = state[target.set_id].as_ref() else {
                continue;
            };
            let input = if self.operation.cached.query_plan[target.set_id].ty_id == target.ty_id {
                InputResponseObjectSet::default().with_response_objects(refs.clone())
            } else {
                InputResponseObjectSet::default().with_filtered_response_objects(
                    self.schema(),
                    target.ty_id,
                    refs.clone(),
                )
            };

            if input.is_empty() {
                continue;
            }

            // to be reworked.
            let target_field = target.field();

            // Now we can execute the hook and propagate any errors.
            match response_modifier.rule {
                ResponseModifierRule::AuthorizedParentEdge {
                    directive_id,
                    definition_id,
                } => {
                    let definition = definition_id.walk(self);
                    let directive = directive_id.walk(self);
                    let input = Arc::new(input);
                    let parents = response.read(
                        input.clone(),
                        // FIXME: Total hack, assumes there is only one authorized directive on the field. Need
                        target_field.required_fields_by_supergraph(),
                    );
                    let result = self
                        .hooks()
                        .authorize_parent_edge_post_execution(definition, parents, directive.metadata())
                        .await;
                    tracing::debug!("Authorized result: {result:#?}");
                    // FIXME: make this efficient
                    let result = match result {
                        Ok(result) => {
                            if result.len() == input.len() {
                                result
                                    .into_iter()
                                    .map(|res| res.map_err(GraphqlError::from))
                                    .collect::<Vec<_>>()
                            } else if result.len() == 1 {
                                let res = result.into_iter().next().unwrap().map_err(GraphqlError::from);
                                (0..input.len()).map(|_| res.clone()).collect()
                            } else {
                                tracing::error!("Incorrect number of authorization replies");
                                (0..input.len())
                                    .map(|_| Err(GraphqlError::new("Authorization failure", ErrorCode::HookError)))
                                    .collect()
                            }
                        }
                        Err(err) => (0..input.len()).map(|_| Err(err.clone())).collect(),
                    };

                    for (obj_ref, result) in input.iter().zip_eq(result) {
                        if let Err(err) = result {
                            // If the current field is required, the error must be propagated upwards,
                            // so the parent object path is enough.
                            if definition.ty().wrapping.is_required() {
                                response.propagate_null(&obj_ref.path);
                            } else {
                                // Otherwise we don't need to propagate anything and just need to mark
                                // the current value as inaccessible. So null for the client, but
                                // available for requirements to be sent to subgraphs.
                                response.make_inacessible(ResponseValueId::Field {
                                    object_id: obj_ref.id,
                                    key: target_field.key(),
                                    nullable: true,
                                });
                            }
                            response.push_error(err.clone().with_path((&obj_ref.path, target_field.response_key)));
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
                        input.clone(),
                        // FIXME: Total hack, assumes there is only one authorized directive on the field. Need
                        target_field.required_fields_by_supergraph(),
                    );
                    let result = self
                        .hooks()
                        .authorize_edge_node_post_execution(definition, nodes, directive.metadata())
                        .await;
                    tracing::debug!("Authorized result: {result:#?}");
                    // FIXME: make this efficient
                    let result = match result {
                        Ok(result) => {
                            if result.len() == input.len() {
                                result
                                    .into_iter()
                                    .map(|res| res.map_err(GraphqlError::from))
                                    .collect::<Vec<_>>()
                            } else if result.len() == 1 {
                                let res = result.into_iter().next().unwrap().map_err(GraphqlError::from);
                                (0..input.len()).map(|_| res.clone()).collect()
                            } else {
                                tracing::error!("Incorrect number of authorization replies");
                                (0..input.len())
                                    .map(|_| Err(GraphqlError::new("Authorization failure", ErrorCode::HookError)))
                                    .collect()
                            }
                        }
                        Err(err) => (0..input.len()).map(|_| Err(err.clone())).collect(),
                    };

                    for (obj_ref, result) in input.iter().zip_eq(result) {
                        if let Err(err) = result {
                            response.propagate_null(&obj_ref.path);
                            response.push_error(err.clone().with_path(&obj_ref.path));
                        }
                    }
                }
            }
        }
    }
}
