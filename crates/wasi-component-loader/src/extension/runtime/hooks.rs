use engine_error::ErrorResponse;
use event_queue::EventQueue;
use http::{request, response};
use runtime::extension::HooksExtension;

use crate::{SharedContext, extension::GatewayWasmExtensions};

impl HooksExtension<SharedContext> for GatewayWasmExtensions {
    async fn on_request(&self, parts: request::Parts) -> Result<(SharedContext, request::Parts), ErrorResponse> {
        let context = SharedContext::new(EventQueue::new(self.hooks_event_filter), None);
        let Some(pool) = self.hooks.as_ref() else {
            return Ok((context, parts));
        };

        let mut instance = pool.get().await.map_err(|e| ErrorResponse {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            errors: vec![engine_error::GraphqlError::new(
                e.to_string(),
                engine_error::ErrorCode::ExtensionError,
            )],
            headers: Default::default(),
        })?;

        let parts = instance
            .on_request(context.clone(), parts)
            .await
            .map_err(|e| e.into_graphql_error_response(engine_error::ErrorCode::ExtensionError))?;

        Ok((context, parts))
    }

    async fn on_response(&self, context: &SharedContext, parts: response::Parts) -> Result<response::Parts, String> {
        let Some(pool) = self.hooks.as_ref() else {
            return Ok(parts);
        };
        let mut instance = pool.get().await.map_err(|e| e.to_string())?;

        instance
            .on_response(context.clone(), parts)
            .await
            .map_err(|e| e.to_string())
    }
}
