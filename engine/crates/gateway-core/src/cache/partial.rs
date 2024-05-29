use std::sync::Arc;

use common_types::auth::ExecutionAuth;
use cynic_parser::ExecutableDocument;
use futures_util::future::join_all;
use partial_caching::CachingPlan;
use runtime::{cache::Cache, context::RequestContext};
use tracing::{info_span, Instrument};

use crate::Executor;

pub async fn partial_caching_execution<Exec, Ctx>(
    plan: CachingPlan,
    cache: &Cache,
    auth: ExecutionAuth,
    mut request: engine::Request,
    executor: &Arc<Exec>,
    ctx: &Arc<Ctx>,
) -> Result<Arc<engine::Response>, Exec::Error>
where
    Exec: Executor<Context = Ctx>,
    Ctx: RequestContext,
{
    let operation_type = operation_type(&plan.document, request.operation_name());

    let mut fetch_phase = plan.start_fetch_phase(&auth, ctx.headers(), &request.variables);
    let cache_keys = fetch_phase.cache_keys();

    let cache_fetches = cache_keys.iter().map(|key| {
        let key = cache.build_key(&key.to_string());
        async move { cache.get_json::<serde_json::Value>(&key).await }
    });

    for (fetch_result, key) in join_all(cache_fetches).await.into_iter().zip(cache_keys) {
        match fetch_result {
            Ok(entry) => fetch_phase.record_cache_entry(&key, entry),
            Err(error) => {
                // We basically just log and then pretend this is a miss
                tracing::warn!("error when fetching from cache: {error}");
            }
        }
    }

    match fetch_phase.finish() {
        partial_caching::FetchPhaseResult::PartialHit(execution_phase) => {
            request.operation_plan_cache_key.query = execution_phase.query();

            let mut executor_response = Arc::clone(executor)
                .execute(Arc::clone(ctx), auth, request)
                .instrument(info_span!("execute"))
                .await?;

            let (merged_data, _updates) = execution_phase.handle_response(executor_response.data);
            executor_response.data = merged_data;

            Ok(Arc::new(executor_response))
        }
        partial_caching::FetchPhaseResult::CompleteHit(hit) => {
            let (data, _updates) = hit.response_and_updates();

            Ok(Arc::new(engine::Response::new(
                data,
                request.operation_name(),
                operation_type,
            )))
        }
    }
}

fn operation_type(document: &ExecutableDocument, operation_name: Option<&str>) -> common_types::OperationType {
    let operation = match operation_name {
        Some(name) => document.operations().find(|op| op.name() == Some(name)),
        None => document.operations().next(),
    };

    let Some(operation) = operation else {
        return common_types::OperationType::Query {
            is_introspection: false,
        };
    };

    match operation.operation_type() {
        cynic_parser::common::OperationType::Query => common_types::OperationType::Query {
            is_introspection: operation.selection_set().all(|selection| match selection {
                cynic_parser::executable::Selection::Field(field) => {
                    field.name().starts_with("__") || field.name() == "_service"
                }
                _ => false,
            }),
        },
        cynic_parser::common::OperationType::Mutation => common_types::OperationType::Mutation,
        cynic_parser::common::OperationType::Subscription => common_types::OperationType::Subscription,
    }
}
