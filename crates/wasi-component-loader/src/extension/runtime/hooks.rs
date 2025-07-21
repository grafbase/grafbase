use engine_error::ErrorResponse;
use event_queue::EventQueue;
use http::{request, response};
use runtime::extension::{HooksExtension, OnRequest};

use crate::{WasmContext, extension::GatewayWasmExtensions, wasmsafe};

impl HooksExtension<WasmContext> for GatewayWasmExtensions {
    async fn on_request(&self, parts: request::Parts) -> Result<OnRequest<WasmContext>, ErrorResponse> {
        let context = WasmContext::new(EventQueue::new(self.hooks_event_filter));
        let Some(pool) = self.hooks.as_ref() else {
            return Ok(OnRequest {
                context,
                parts,
                contract_key: None,
            });
        };

        let mut instance = pool.get().await.map_err(|err| {
            tracing::error!("Failed to get instance from pool: {err}");
            ErrorResponse::internal_extension_error()
        })?;

        wasmsafe!(instance.on_request(&context, parts).await).map(|parts| OnRequest {
            context,
            parts,
            contract_key: None,
        })
    }

    async fn on_response(&self, context: WasmContext, parts: response::Parts) -> Result<response::Parts, String> {
        let Some(pool) = self.hooks.as_ref() else {
            return Ok(parts);
        };
        let mut instance = pool.get().await.map_err(|e| e.to_string())?;

        wasmsafe!(instance.on_response(context, parts).await)
    }
}
