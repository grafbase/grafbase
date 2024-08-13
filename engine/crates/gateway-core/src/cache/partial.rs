use std::{sync::Arc, time::Duration};

use common_types::auth::ExecutionAuth;
use cynic_parser::ExecutableDocument;
use engine::{InitialResponse, StreamingPayload};
use futures_channel::mpsc;
use futures_util::{
    future::join_all,
    stream::{self, BoxStream, StreamExt},
    FutureExt, SinkExt,
};
use partial_caching::{CacheUpdatePhase, CachingPlan, FetchPhaseResult, StreamingExecutionPhase, TypeRelationships};
use registry_for_cache::PartialCacheRegistry;
use runtime::{
    cache::{Cache, CacheMetadata, EntryState},
    context::RequestContext,
};
use tracing::{info_span, Instrument};

use crate::Executor;

pub async fn partial_caching_execution<Exec, Ctx>(
    plan: CachingPlan,
    cache: &Cache,
    auth: ExecutionAuth,
    mut request: engine::Request,
    executor: &Arc<Exec>,
    ctx: &Arc<Ctx>,
    registry: &Arc<PartialCacheRegistry>,
) -> Result<Arc<engine::Response>, Exec::Error>
where
    Exec: Executor<Context = Ctx>,
    Ctx: RequestContext,
{
    let operation_type = operation_type(&plan.document, request.operation_name());

    match run_fetch_phase(plan, &request, &auth, ctx, cache, registry).await {
        partial_caching::FetchPhaseResult::PartialHit(execution_phase) => {
            request.query = execution_phase.query();

            let mut executor_response = Arc::clone(executor)
                .execute(Arc::clone(ctx), auth, request)
                .instrument(info_span!("execute"))
                .await?;

            let (merged_data, update_phase) =
                execution_phase.handle_full_response(executor_response.data, !executor_response.errors.is_empty());

            let partial_caching::Response { body, headers } = merged_data;

            executor_response.data = body;
            executor_response.http_headers.extend(headers);

            if let Some(update_phase) = update_phase {
                ctx.wait_until(run_update_phase(update_phase, cache.clone()).boxed())
                    .await;
            }

            Ok(Arc::new(executor_response))
        }
        partial_caching::FetchPhaseResult::CompleteHit(hit) => {
            let (response, update_phase) = hit.response_and_updates();
            let partial_caching::Response { body, headers } = response;

            if let Some(update_phase) = update_phase {
                ctx.wait_until(run_update_phase(update_phase, cache.clone()).boxed())
                    .await;
            }

            Ok(Arc::new(
                engine::Response::new(
                    body,
                    engine::GraphqlOperationAnalyticsAttributes {
                        name: request.operation_name().map(str::to_string),
                        name_or_generated_one: request.operation_name().map(str::to_string).unwrap_or_default(),
                        r#type: operation_type,
                        used_fields: String::new(),
                    },
                )
                .http_headers(headers),
            ))
        }
    }
}

pub async fn partial_caching_stream<Exec, Ctx>(
    plan: CachingPlan,
    cache: &Cache,
    auth: ExecutionAuth,
    mut request: engine::Request,
    executor: &Arc<Exec>,
    ctx: &Arc<Ctx>,
    registry: &Arc<PartialCacheRegistry>,
) -> Result<BoxStream<'static, engine::StreamingPayload>, Exec::Error>
where
    Exec: Executor<Context = Ctx>,
    Ctx: RequestContext,
{
    let operation_type = operation_type(&plan.document, request.operation_name());

    match run_fetch_phase(plan, &request, &auth, ctx, cache, registry).await {
        FetchPhaseResult::PartialHit(execution_phase) => {
            let deferred_execution_phase = execution_phase.streaming();
            request.query = deferred_execution_phase.query();

            let engine_stream = Arc::clone(executor)
                .execute_stream_v2(Arc::clone(ctx), auth, request)
                .instrument(info_span!("execute"))
                .await?;

            let (response_sender, response_receiver) = mpsc::channel(5);

            ctx.wait_until(
                run_execution_phase_stream(deferred_execution_phase, engine_stream, response_sender, cache.clone())
                    .boxed(),
            )
            .await;

            Ok(Box::pin(response_receiver))
        }
        partial_caching::FetchPhaseResult::CompleteHit(hit) => {
            let (response, update_phase) = hit.response_and_updates();
            let partial_caching::Response { body, headers } = response;

            if let Some(update_phase) = update_phase {
                ctx.wait_until(run_update_phase(update_phase, cache.clone()).boxed())
                    .await;
            }

            let response = engine::Response::new(
                body,
                engine::GraphqlOperationAnalyticsAttributes {
                    name: request.operation_name().map(str::to_string),
                    name_or_generated_one: request.operation_name().map(str::to_string).unwrap_or_default(),
                    r#type: operation_type,
                    used_fields: String::new(),
                },
            )
            .http_headers(headers);

            Ok(stream::once(async move {
                engine::StreamingPayload::InitialResponse(InitialResponse {
                    data: response.data.into_compact_value(),
                    errors: response.errors,
                    has_next: false,
                })
            })
            .boxed())
        }
    }
}

async fn run_execution_phase_stream(
    mut execution_phase: StreamingExecutionPhase,
    mut engine_stream: BoxStream<'static, engine::StreamingPayload>,
    mut response_sender: mpsc::Sender<engine::StreamingPayload>,
    cache: Cache,
) {
    let Some(StreamingPayload::InitialResponse(mut payload)) = engine_stream.next().await else {
        todo!("GB-6966");
    };

    if let Some(data) = payload.data {
        payload.data = Some(execution_phase.record_initial_response(data, !payload.errors.is_empty()));
    }

    if response_sender
        .send(StreamingPayload::InitialResponse(payload))
        .await
        .is_err()
    {
        return;
    };

    while let Some(next_chunk) = engine_stream.next().await {
        let StreamingPayload::Incremental(mut payload) = next_chunk else {
            todo!("GB-6966");
        };

        let path = payload.path.iter().collect::<Vec<_>>();

        payload.data = execution_phase.record_incremental_response(
            payload.label.as_deref(),
            &path,
            payload.data,
            !payload.errors.is_empty(),
        );

        if response_sender
            .send(StreamingPayload::Incremental(payload))
            .await
            .is_err()
        {
            return;
        };
    }

    if let Some(update_phase) = execution_phase.finish() {
        run_update_phase(update_phase, cache).await;
    }
}

async fn run_fetch_phase<Ctx>(
    plan: CachingPlan,
    request: &engine::Request,
    auth: &ExecutionAuth,
    ctx: &Arc<Ctx>,
    cache: &Cache,
    registry: &Arc<PartialCacheRegistry>,
) -> FetchPhaseResult
where
    Ctx: RequestContext,
{
    let mut fetch_phase = plan.start_fetch_phase(auth, ctx.headers(), &request.variables);
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

    fetch_phase.finish(Arc::clone(registry) as Arc<dyn TypeRelationships>)
}

async fn run_update_phase(update_phase: CacheUpdatePhase, cache: Cache) {
    // Unsure whether I should run these in parallel or series.
    // Series will be slower and hogs memory for longer, but in parallel will likely hog the CPU
    // for longer, which as we know CF does not like.  Sigh.  For now, I'll write it serially.
    for update in update_phase.updates() {
        let key = cache.build_key(update.key);

        let metadata = CacheMetadata {
            max_age: Duration::from_secs(update.cache_control.max_age as u64),
            stale_while_revalidate: Duration::from_secs(update.cache_control.stale_while_revalidate as u64),
            tags: vec![],
            should_purge_related: false,
            should_cache: true,
        };

        if let Err(err) = cache
            .put_json(&key, EntryState::Fresh, &update, metadata)
            .instrument(tracing::info_span!("cache_put"))
            .await
        {
            tracing::error!("Error cache PUT: {}", err);
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
