use std::sync::Arc;

use axum::{extract::State, Json};

use crate::types::ServerMessage;

use super::{
    errors::ApiError,
    server::HandlerState,
    types::{LogEvent, LogEventType},
};

pub async fn log_event_endpoint(
    State(handler_state): State<Arc<HandlerState>>,
    Json(request): Json<LogEvent>,
) -> Result<(), ApiError> {
    let LogEvent { request_id, r#type } = request;
    let message = match r#type {
        LogEventType::OperationStarted { name } => ServerMessage::OperationStarted { request_id, name },
        LogEventType::OperationCompleted { name, duration, r#type } => ServerMessage::OperationCompleted {
            request_id,
            name,
            duration,
            r#type,
        },
    };

    handler_state.bridge_sender.send(message).await.unwrap();

    Ok(())
}
