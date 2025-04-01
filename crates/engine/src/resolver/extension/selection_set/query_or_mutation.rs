use std::sync::Arc;

use error::GraphqlError;
use futures::FutureExt as _;
use runtime::extension::{Data, SelectionSetResolverExtension};
use schema::InputValueSet;
use walker::Walk;

use crate::{
    Runtime,
    execution::{ExecutionContext, ExecutionResult},
    prepare::Plan,
    response::{InputResponseObjectSet, SubgraphResponse},
};

impl super::SelectionSetResolverExtension {
    pub(in crate::resolver) async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        input_object_refs: Arc<InputResponseObjectSet>,
        mut subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<SubgraphResponse> {
        let definition = self.definition.walk(&ctx);
        let subgraph_headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());
        let field = plan.get_field(self.field_id);

        let result = ctx
            .runtime()
            .extensions()
            .resolve_query_or_mutation_field(
                definition.extension_id,
                definition.subgraph().into(),
                &self.prepared_data,
                subgraph_headers,
                self.arguments
                    .iter()
                    .map(|(id, args)| (*id, args.walk(&ctx).view(&InputValueSet::All, ctx.variables()))),
            )
            .boxed()
            .await;

        let response = subgraph_response.as_shared_mut();

        let input_object_id = input_object_refs.ids().next().ok_or("No object to update")?;
        match result {
            Ok(data) => match data {
                Data::JsonBytes(bytes) => {
                    tracing::debug!("Received:\n{}", String::from_utf8_lossy(&bytes));

                    response
                        .seed(&ctx, input_object_id)
                        .deserialize_field_as_entity(
                            field.subgraph_response_key_str(),
                            &mut sonic_rs::Deserializer::from_slice(&bytes),
                        )
                        .map_err(|err| {
                            tracing::error!("Failed to deserialize subgraph response: {}", err);
                            GraphqlError::invalid_subgraph_response()
                                .with_location(field.location())
                                .with_path(&input_object_refs[input_object_id].path)
                        })?;
                }
                Data::CborBytes(bytes) => {
                    tracing::debug!(
                        "Received:\n{}",
                        minicbor_serde::from_slice(&bytes)
                            .ok()
                            .and_then(|v: sonic_rs::Value| sonic_rs::to_string_pretty(&v).ok())
                            .unwrap_or_else(|| "<error>".to_string())
                    );

                    response
                        .seed(&ctx, input_object_id)
                        .deserialize_field_as_entity(
                            field.subgraph_response_key_str(),
                            &mut minicbor_serde::Deserializer::new(&bytes),
                        )
                        .map_err(|err| {
                            tracing::error!("Failed to deserialize subgraph response: {}", err);
                            GraphqlError::invalid_subgraph_response()
                                .with_location(field.location())
                                .with_path(&input_object_refs[input_object_id].path)
                        })?;
                }
            },
            Err(err) => {
                response.borrow_mut().insert_errors(
                    err.with_location(field.location())
                        .with_path(&input_object_refs[input_object_id].path),
                    [input_object_id],
                );
            }
        }

        Ok(subgraph_response)
    }
}
