use std::sync::Arc;

use axum::{extract::State, Json};

use super::{errors::ApiError, server::HandlerState, types::LogEvent};

#[allow(clippy::unused_async)]
pub async fn log_event_endpoint(
    State(handler_state): State<Arc<HandlerState>>,
    Json(request): Json<LogEvent>,
) -> Result<(), ApiError> {
    use super::types::LogEventType as InputLogEventType;
    use crate::types::LogEventType as OutputLogEventType;
    use crate::types::{RequestCompletedOutcome, ServerMessage};

    let LogEvent { request_id, r#type } = request;
    let message = ServerMessage::RequestScopedMessage {
        request_id,
        event_type: match r#type {
            InputLogEventType::OperationStarted { name: _ } => return Ok(()), // Ignore for now.
            InputLogEventType::OperationCompleted { name, duration, r#type } => OutputLogEventType::RequestCompleted {
                name,
                duration,
                request_completed_type: RequestCompletedOutcome::Success { r#type },
            },
            InputLogEventType::BadRequest { name, duration } => OutputLogEventType::RequestCompleted {
                name,
                duration,
                request_completed_type: RequestCompletedOutcome::BadRequest,
            },
            InputLogEventType::NestedRequest {
                url,
                method,
                status_code,
                duration,
                body,
                content_type,
            } => OutputLogEventType::NestedEvent(crate::types::NestedRequestScopedMessage::NestedRequest {
                url,
                method,
                status_code,
                duration,
                body,
                content_type,
            }),
        },
    };
    handler_state.message_sender.send(message).unwrap();

    Ok(())
}
