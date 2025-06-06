use engine_error::ErrorResponse;
use http::{request, response};
use runtime::extension::HooksExtension;

use crate::{SharedContext, extension::WasmHooks};

impl HooksExtension for WasmHooks {
    type Context = SharedContext;

    async fn on_request(&self, parts: request::Parts) -> Result<request::Parts, ErrorResponse> {
        let Some(pool) = self.pool() else { return Ok(parts) };

        let mut instance = pool.get().await.map_err(|e| ErrorResponse {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            errors: vec![engine_error::GraphqlError::new(
                e.to_string(),
                engine_error::ErrorCode::ExtensionError,
            )],
        })?;

        instance.on_request(parts).await.map_err(|e| match e {
            crate::ErrorResponse::Internal(err) => ErrorResponse {
                status: http::StatusCode::INTERNAL_SERVER_ERROR,
                errors: vec![engine_error::GraphqlError::new(
                    err.to_string(),
                    engine_error::ErrorCode::ExtensionError,
                )],
            },
            crate::ErrorResponse::Guest(err) => {
                dbg!(1);
                err.into_graphql_error_response(engine_error::ErrorCode::ExtensionError)
            }
        })
    }

    async fn on_response(&self, parts: response::Parts) -> Result<response::Parts, ErrorResponse> {
        let Some(pool) = self.pool() else { return Ok(parts) };

        let mut instance = pool.get().await.map_err(|e| ErrorResponse {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            errors: vec![engine_error::GraphqlError::new(
                e.to_string(),
                engine_error::ErrorCode::ExtensionError,
            )],
        })?;

        instance.on_response(parts).await.map_err(|e| match e {
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
}
