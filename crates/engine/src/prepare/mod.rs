pub(crate) mod cached;
mod context;
mod operation_plan;
mod trusted_documents;
mod with_cache;
mod without_cache;

use std::sync::Arc;

pub(crate) use cached::*;
pub(crate) use context::*;
use operation::Operation;
pub(crate) use operation_plan::*;

use ::operation::{ComplexityCost, Request, Variables};
use futures::FutureExt;
use grafbase_telemetry::graphql::{GraphqlOperationAttributes, OperationName, OperationType};
use runtime::{hooks::Hooks, operation_cache::OperationCache};
use tracing::{info_span, Instrument};

use crate::{
    engine::cache::CacheKey,
    response::{GraphqlError, Response},
    Engine, ErrorCode, Runtime,
};

impl<R: Runtime> Engine<R> {
    pub(crate) fn warm_operation(&self, document: OperationDocument<'_>) -> Result<CachedOperation, String> {
        let operation = Operation::parse(&self.schema, document.operation_name(), &document.content)
            .map_err(|err| err.to_string())?;
        crate::prepare::solve(&self.schema, document, operation).map_err(|err| err.to_string())
    }
}

impl<R: Runtime> PrepareContext<'_, R> {
    pub(crate) async fn prepare_operation(
        &mut self,
        request: Request,
    ) -> Result<PreparedOperation, Response<<R::Hooks as Hooks>::OnOperationResponseOutput>> {
        let span = info_span!("prepare operation");
        let result = self.prepare_operation_inner(request).instrument(span).await;
        let duration = self.executed_operation_builder.track_prepare();

        match result {
            Ok(operation) => {
                self.metrics()
                    .record_successful_preparation_duration(operation.attributes(), duration);

                Ok(operation)
            }
            Err(response) => {
                self.metrics()
                    .record_failed_preparation_duration(response.operation_attributes().cloned(), duration);

                Err(response)
            }
        }
    }

    pub(super) async fn prepare_operation_inner(
        &mut self,
        mut request: Request,
    ) -> Result<PreparedOperation, Response<<R::Hooks as Hooks>::OnOperationResponseOutput>> {
        let variables = std::mem::take(&mut request.variables);
        let cache_result = {
            let extracted = match self.extract_operation_document(&request) {
                Ok(doc) => doc,
                // If we have an error a this stage, it means we couldn't determine what document
                // to load, so we don't consider it a well-formed GraphQL-over-HTTP request.
                Err(err) => return Err(Response::refuse_request_with(http::StatusCode::BAD_REQUEST, vec![err])),
            };

            let cache_key = CacheKey::document(self.schema(), &extracted.key);
            if let Some(operation) = self.operation_cache().get(&cache_key).await {
                self.executed_operation_builder.set_cached_plan();
                self.metrics().record_operation_cache_hit();

                OpCache::Hit(operation)
            } else {
                self.metrics().record_operation_cache_miss();
                match extracted.into_operation_document().await {
                    Ok(document) => OpCache::Miss { cache_key, document },
                    Err(err) => return Err(Response::request_error(None, [err])),
                }
            }
        };

        match cache_result {
            OpCache::Hit(cached) => self.prepare_operation_with_cache(cached, variables).await,
            OpCache::Miss { cache_key, document } => {
                let prepared = self.prepare_operation_without_cache(document, variables).await?;

                let cache_fut = self.operation_cache().insert(cache_key, prepared.cached.clone());
                self.push_background_future(cache_fut.boxed());

                Ok(prepared)
            }
        }
    }
}

enum OpCache<'a> {
    Hit(Arc<CachedOperation>),
    Miss {
        cache_key: String,
        document: OperationDocument<'a>,
    },
}

/// The set of Operation attributes that can be cached
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CachedOperationAttributes {
    pub ty: OperationType,
    pub name: OperationName,
    pub sanitized_query: Arc<str>,
}

pub(crate) struct PreparedOperation {
    pub cached: Arc<CachedOperation>,
    pub plan: OperationPlan,
    pub variables: Variables,
    pub complexity_cost: Option<ComplexityCost>,
}

impl PreparedOperation {
    pub fn attributes(&self) -> GraphqlOperationAttributes {
        self.cached
            .operation
            .attributes
            .clone()
            .with_complexity_cost(self.complexity_cost)
    }
}

fn mutation_not_allowed_with_safe_method<OnOperationResponseHookOutput>() -> Response<OnOperationResponseHookOutput> {
    Response::refuse_request_with(
        http::StatusCode::METHOD_NOT_ALLOWED,
        vec![GraphqlError::new(
            "Mutation is not allowed with a safe method like GET",
            ErrorCode::BadRequest,
        )],
    )
}
