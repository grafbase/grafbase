use std::sync::Arc;

use runtime::operation_cache::OperationCache;

use crate::{prepare::PrepareContext, CachedOperation};

use super::{
    cache::{DocumentKey, Key},
    EarlyHttpContext, Engine, Runtime,
};

pub struct PrewarmOperation {
    document: String,
    operation_name: Option<String>,
    document_key: DocumentKey<'static>,
}

impl<R: Runtime> Engine<R> {
    pub async fn prewarm(self: &Arc<Self>, operations: Vec<PrewarmOperation>) {
        if operations.is_empty() {
            return;
        }
        tracing::info!("Prewarming {} operations", operations.len());

        let Ok((request_context, hooks_context)) = self
            .create_request_context(
                &EarlyHttpContext {
                    method: http::Method::POST,
                    response_format: super::ResponseFormat::application_json(),
                    include_grafbase_response_extension: false,
                },
                Default::default(),
            )
            .await
        else {
            tracing::error!("Couldn't construct prewarming context, skipping prewarm");
            return;
        };

        let request_context = Arc::new(request_context);

        for operation in operations {
            let name = operation.operation_name.as_deref();
            let prepare_context = PrepareContext::new(self, &request_context, hooks_context.clone());
            let result = prepare_context.build_cached_operation(
                name,
                operation.document.as_str(),
                operation.document_key.clone(),
            );

            match result {
                Ok(cached_operation) => {
                    let cache_key = Key::Operation {
                        name,
                        schema: &self.schema,
                        document: operation.document_key,
                    }
                    .to_string();

                    self.runtime
                        .operation_cache()
                        .insert(cache_key, Arc::new(cached_operation))
                        .await;
                }
                Err(err) => {
                    tracing::warn!(
                        "Could not plan operation {}: {err}",
                        operation.operation_name.unwrap_or_default()
                    );
                }
            }

            futures_lite::future::yield_now().await
        }
        tracing::info!("Prewarming finished");
    }
}

impl PrewarmOperation {
    pub fn new(op: impl AsRef<CachedOperation>) -> Self {
        let op = op.as_ref();
        PrewarmOperation {
            document: op.document.clone(),
            operation_name: op.operation_name.as_ref().map(ToString::to_string),
            document_key: op.document_key.clone(),
        }
    }
}
