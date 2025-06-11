use engine_error::ErrorResponse;
use event_queue::EventQueue;
use http::{request, response};
use runtime::extension::HooksExtension;

use crate::{SharedContext, extension::WasmHooks};

impl HooksExtension for WasmHooks {
    type Context = SharedContext;

    fn new_context(&self) -> Self::Context {
        let event_queue = EventQueue::new(self.extension());

        SharedContext {
            event_queue,
            ..Default::default()
        }
    }

    async fn on_request(&self, context: Self::Context, parts: request::Parts) -> Result<request::Parts, ErrorResponse> {
        let Some(pool) = self.pool() else { return Ok(parts) };

        let mut instance = pool.get().await.map_err(|e| ErrorResponse {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            errors: vec![engine_error::GraphqlError::new(
                e.to_string(),
                engine_error::ErrorCode::ExtensionError,
            )],
        })?;

        instance.on_request(context, parts).await.map_err(|e| match e {
            crate::ErrorResponse::Internal(err) => ErrorResponse {
                status: http::StatusCode::INTERNAL_SERVER_ERROR,
                errors: vec![engine_error::GraphqlError::new(
                    err.to_string(),
                    engine_error::ErrorCode::ExtensionError,
                )],
            },
            crate::ErrorResponse::Guest(err) => {
                err.into_graphql_error_response(engine_error::ErrorCode::ExtensionError)
            }
        })
    }

    async fn on_response(&self, context: Self::Context, parts: response::Parts) -> Result<response::Parts, String> {
        let Some(pool) = self.pool() else { return Ok(parts) };

        let mut instance = pool.get().await.map_err(|e| e.to_string())?;
        instance.on_response(context, parts).await.map_err(|e| e.to_string())
    }
}
