use futures::FutureExt as _;
use itertools::Itertools;
use runtime::extension::{AuthorizationDecisions, AuthorizationExtension as _};
use schema::DirectiveSiteId;
use walker::Walk;

use crate::{
    Runtime,
    prepare::{
        PlanFieldArguments, ResponseModifier, ResponseModifierRule, ResponseModifierRuleTarget,
        create_extension_directive_response_view,
    },
    response::{ErrorCode, GraphqlError, ParentObjectSet, ResponseBuilder, ResponseValueId},
};

use super::{ExecutionContext, state::OperationExecutionState};

impl<'ctx, R: Runtime> ExecutionContext<'ctx, R> {
    pub(super) async fn execute_response_modifier(
        &self,
        state: &mut OperationExecutionState<'ctx, R>,
        response: &mut ResponseBuilder<'ctx>,
        response_modifier: ResponseModifier<'ctx>,
    ) {
        for target in response_modifier.sorted_targets() {
            let Some(refs) = state[target.set_id].as_ref() else {
                continue;
            };
            let parent_objects = if self.operation.cached.query_plan[target.set_id].ty_id == target.ty_id {
                ParentObjectSet::default().with_response_objects(refs.clone())
            } else {
                ParentObjectSet::default().with_filtered_response_objects(self.schema(), target.ty_id, refs.clone())
            };

            if parent_objects.is_empty() {
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
                    let parents = response.read(
                        parent_objects,
                        // FIXME: Total hack, assumes there is only one authorized directive on the field. Need
                        target_field.required_fields_by_supergraph(),
                    );
                    let result = self
                        .hooks()
                        .authorize_parent_edge_post_execution(definition, &parents, directive.metadata())
                        .await;
                    let parent_objects = parents.into_object_set();
                    tracing::debug!("Authorized result: {result:#?}");
                    // FIXME: make this efficient
                    let result = match result {
                        Ok(result) => {
                            if result.len() == parent_objects.len() {
                                result
                            } else if result.len() == 1 {
                                let res = result.into_iter().next().unwrap();
                                (0..parent_objects.len()).map(|_| res.clone()).collect()
                            } else {
                                tracing::error!("Incorrect number of authorization replies");
                                (0..parent_objects.len())
                                    .map(|_| Err(GraphqlError::new("Authorization failure", ErrorCode::HookError)))
                                    .collect()
                            }
                        }
                        Err(err) => (0..parent_objects.len()).map(|_| Err(err.clone())).collect(),
                    };

                    for (obj_ref, result) in parent_objects.iter().zip_eq(result) {
                        if let Err(err) = result {
                            // If the current field is required, the error must be propagated upwards,
                            // so the parent object path is enough.
                            if definition.ty().wrapping.is_non_null() {
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
                    let nodes = response.read(
                        parent_objects,
                        // FIXME: Total hack, assumes there is only one authorized directive on the field. Need
                        target_field.required_fields_by_supergraph(),
                    );
                    let result = self
                        .hooks()
                        .authorize_edge_node_post_execution(definition, &nodes, directive.metadata())
                        .await;
                    let parent_objects = nodes.into_object_set();
                    tracing::debug!("Authorized result: {result:#?}");
                    // FIXME: make this efficient
                    let result = match result {
                        Ok(result) => {
                            if result.len() == parent_objects.len() {
                                result
                            } else if result.len() == 1 {
                                let res = result.into_iter().next().unwrap();
                                (0..parent_objects.len()).map(|_| res.clone()).collect()
                            } else {
                                tracing::error!("Incorrect number of authorization replies");
                                (0..parent_objects.len())
                                    .map(|_| Err(GraphqlError::new("Authorization failure", ErrorCode::HookError)))
                                    .collect()
                            }
                        }
                        Err(err) => (0..parent_objects.len()).map(|_| Err(err.clone())).collect(),
                    };

                    for (obj_ref, result) in parent_objects.iter().zip_eq(result) {
                        if let Err(err) = result {
                            response.propagate_null(&obj_ref.path);
                            response.push_error(err.clone().with_path(&obj_ref.path));
                        }
                    }
                }
                ResponseModifierRule::Extension {
                    directive_id,
                    target:
                        rule_target @ (ResponseModifierRuleTarget::Field(_, _)
                        | ResponseModifierRuleTarget::FieldParentEntity(_)),
                } => {
                    let field_argument_ids = match rule_target {
                        ResponseModifierRuleTarget::Field(_, field_argument_ids) => field_argument_ids,
                        _ => Default::default(),
                    };
                    let parents = response.read(parent_objects, target_field.required_fields_by_supergraph());

                    let response_view = create_extension_directive_response_view(
                        self.schema(),
                        directive_id.walk(self),
                        field_argument_ids.walk(self),
                        self.variables(),
                        &parents,
                    );

                    let directive = directive_id.walk(self);
                    let result = self
                        .extensions()
                        .authorize_response(
                            &self.request_context.extension_context,
                            directive.extension_id,
                            directive.name(),
                            DirectiveSiteId::from(rule_target).walk(self),
                            response_view.iter(),
                        )
                        // FIXME: Unfortunately, boxing seems to be the only solution for the bug explained here:
                        //        https://github.com/rust-lang/rust/issues/110338#issuecomment-1513761297
                        .boxed()
                        .await;

                    let parent_objects = parents.into_object_set();
                    match result {
                        Ok(AuthorizationDecisions::GrantAll) => (),
                        Ok(AuthorizationDecisions::DenySome {
                            element_to_error,
                            errors,
                        }) => {
                            // If the current field is required, the error must be propagated upwards,
                            // so the parent object path is enough.
                            if target_field.definition().ty().wrapping.is_non_null() {
                                for (element_ix, error_ix) in element_to_error {
                                    let obj_ref = &parent_objects[element_ix as usize];
                                    let err = errors[error_ix as usize].clone();
                                    response.propagate_null(&obj_ref.path);
                                    response.push_error(
                                        err.clone()
                                            .with_path((&obj_ref.path, target_field.response_key))
                                            .with_location(target_field.location),
                                    );
                                }
                            } else {
                                // Otherwise we don't need to propagate anything and just need to mark
                                // the current value as inaccessible. So null for the client, but
                                // available for requirements to be sent to subgraphs.
                                for (element_ix, error_ix) in element_to_error {
                                    let obj_ref = &parent_objects[element_ix as usize];
                                    let err = errors[error_ix as usize].clone();
                                    response.make_inacessible(ResponseValueId::Field {
                                        object_id: obj_ref.id,
                                        key: target_field.key(),
                                        nullable: true,
                                    });
                                    response.push_error(
                                        err.clone()
                                            .with_path((&obj_ref.path, target_field.response_key))
                                            .with_location(target_field.location),
                                    );
                                }
                            }
                        }
                        Ok(AuthorizationDecisions::DenyAll(err)) | Err(err) => {
                            // If the current field is required, the error must be propagated upwards,
                            // so the parent object path is enough.
                            if target_field.definition().ty().wrapping.is_non_null() {
                                for obj_ref in parent_objects.iter() {
                                    response.propagate_null(&obj_ref.path);
                                    response.push_error(
                                        err.clone()
                                            .with_path((&obj_ref.path, target_field.response_key))
                                            .with_location(target_field.location),
                                    );
                                }
                            } else {
                                // Otherwise we don't need to propagate anything and just need to mark
                                // the current value as inaccessible. So null for the client, but
                                // available for requirements to be sent to subgraphs.
                                for obj_ref in parent_objects.iter() {
                                    response.make_inacessible(ResponseValueId::Field {
                                        object_id: obj_ref.id,
                                        key: target_field.key(),
                                        nullable: true,
                                    });
                                    response.push_error(
                                        err.clone()
                                            .with_path((&obj_ref.path, target_field.response_key))
                                            .with_location(target_field.location),
                                    );
                                }
                            }
                        }
                    }
                }
                ResponseModifierRule::Extension {
                    directive_id,
                    target: ResponseModifierRuleTarget::FieldOutput(rule_target),
                } => {
                    let nodes = response.read(
                        parent_objects,
                        // FIXME: Total hack, assumes there is only one authorized directive on the field. Need
                        target_field.required_fields_by_supergraph(),
                    );

                    let response_view = create_extension_directive_response_view(
                        self.schema(),
                        directive_id.walk(self),
                        PlanFieldArguments::empty(self.into()),
                        self.variables(),
                        &nodes,
                    );

                    let directive = directive_id.walk(self);
                    let result = self
                        .extensions()
                        .authorize_response(
                            &self.request_context.extension_context,
                            directive.extension_id,
                            directive.name(),
                            DirectiveSiteId::from(rule_target).walk(self),
                            response_view.iter(),
                        )
                        // FIXME: Unfortunately, boxing seems to be the only solution for the bug explained here:
                        //        https://github.com/rust-lang/rust/issues/110338#issuecomment-1513761297
                        .boxed()
                        .await;
                    tracing::debug!("Response authorization: {result:?}");

                    let parent_objects = nodes.into_object_set();
                    match result {
                        Ok(AuthorizationDecisions::GrantAll) => (),
                        Ok(AuthorizationDecisions::DenySome {
                            element_to_error,
                            errors,
                        }) => {
                            for (element_ix, error_ix) in element_to_error {
                                let obj_ref = &parent_objects[element_ix as usize];
                                let err = errors[error_ix as usize].clone();
                                response.propagate_null(&obj_ref.path);
                                response.push_error(
                                    err.clone()
                                        .with_path(&obj_ref.path)
                                        .with_location(target_field.location),
                                );
                            }
                        }
                        Ok(AuthorizationDecisions::DenyAll(err)) | Err(err) => {
                            for obj_ref in parent_objects.iter() {
                                response.propagate_null(&obj_ref.path);
                                response.push_error(
                                    err.clone()
                                        .with_path(&obj_ref.path)
                                        .with_location(target_field.location),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
