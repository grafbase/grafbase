use std::sync::Arc;

use futures_lite::FutureExt;
use runtime::{
    extension::{Data, ExtensionDirective, ExtensionRuntime},
    hooks::EdgeDefinition,
};
use schema::{FieldResolverExtensionDefinition, FieldResolverExtensionDefinitionRecord};
use walker::Walk;

use crate::{
    execution::{ExecutionContext, ExecutionResult},
    prepare::{Plan, SubgraphField},
    resolver::Resolver,
    response::{GraphqlError, InputResponseObjectSet, ResponseObjectsView, SubgraphResponse},
    Runtime,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct FieldResolverExtension {
    pub definition: FieldResolverExtensionDefinitionRecord,
}

impl FieldResolverExtension {
    pub(in crate::resolver) fn prepare(definition: FieldResolverExtensionDefinition<'_>) -> Resolver {
        Resolver::FieldResolverExtension(Self {
            definition: *definition,
        })
    }

    pub(in crate::resolver) fn prepare_request<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        root_response_objects: ResponseObjectsView<'_>,
        subgraph_response: SubgraphResponse,
    ) -> FieldResolverExtensionRequest<'ctx> {
        let input_object_refs = root_response_objects.into_input_object_refs();
        let directive = self.definition.walk(ctx.schema()).directive();
        let field_definition = plan
            .selection_set()
            .fields()
            .next()
            .expect("At least one field must be present");
        FieldResolverExtensionRequest {
            directive,
            field: field_definition,
            subgraph_response,
            input_object_refs,
        }
    }
}

pub(in crate::resolver) struct FieldResolverExtensionRequest<'ctx> {
    directive: schema::ExtensionDirective<'ctx>,
    field: SubgraphField<'ctx>,
    subgraph_response: SubgraphResponse,
    input_object_refs: Arc<InputResponseObjectSet>,
}

impl<'ctx> FieldResolverExtensionRequest<'ctx> {
    pub async fn execute<R: Runtime>(self, ctx: ExecutionContext<'ctx, R>) -> ExecutionResult<SubgraphResponse> {
        let Self {
            directive,
            field,
            mut subgraph_response,
            input_object_refs,
        } = self;

        let field_definition = field.definition();
        let result = ctx
            .engine
            .runtime
            .extensions()
            .resolve_field(
                directive.extension_id,
                directive.subgraph(),
                &Default::default(),
                EdgeDefinition {
                    parent_type_name: field_definition.parent_entity().name(),
                    field_name: field_definition.name(),
                },
                ExtensionDirective {
                    name: directive.name(),
                    static_arguments: directive.static_arguments(),
                },
                (0..input_object_refs.len()).map(|_| serde_json::json!({})),
            )
            // FIXME: Unfortunately, boxing seems to be the only solution for the bug explained here:
            //        https://github.com/rust-lang/rust/issues/110338#issuecomment-1513761297
            //        Otherwise is not correctly evaluated to be Send due with the associated
            //        return type.
            .boxed()
            .await;

        match result {
            Ok(result) => {
                let response = subgraph_response.as_shared_mut();
                for (id, result) in input_object_refs.ids().zip(result) {
                    match result {
                        Ok(data) => match data {
                            Data::JsonBytes(bytes) => {
                                response
                                    .seed(&ctx, id)
                                    .deserialize_field_as_entity(
                                        field.subgraph_response_key_str(),
                                        &mut serde_json::Deserializer::from_slice(&bytes),
                                    )
                                    .map_err(|err| {
                                        tracing::error!("Failed to deserialize subgraph response: {}", err);
                                        GraphqlError::invalid_subgraph_response()
                                    })?;
                            }
                            Data::CborBytes(bytes) => {
                                response
                                    .seed(&ctx, id)
                                    .deserialize_field_as_entity(
                                        field.subgraph_response_key_str(),
                                        &mut minicbor_serde::Deserializer::new(&bytes),
                                    )
                                    .map_err(|err| {
                                        tracing::error!("Failed to deserialize subgraph response: {}", err);
                                        GraphqlError::invalid_subgraph_response()
                                    })?;
                            }
                        },
                        Err(err) => response.borrow_mut().insert_errors(err, [id]),
                    }
                }
            }
            Err(err) => subgraph_response.set_subgraph_errors(vec![err.into()]),
        }

        Ok(subgraph_response)
    }
}
