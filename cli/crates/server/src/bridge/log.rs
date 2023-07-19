use std::sync::Arc;

use axum::{extract::State, Json};

use crate::types::ServerMessage;

use super::{errors::ApiError, server::HandlerState, types::LogEvent};

pub async fn log_event_endpoint(
    State(handler_state): State<Arc<HandlerState>>,
    Json(request): Json<LogEvent>,
) -> Result<(), ApiError> {
    let LogEvent { request_id, r#type } = request;
    let message = ServerMessage::OperationLogMessage {
        request_id,
        event_type: r#type,
    };
    handler_state.bridge_sender.send(message).await.unwrap();

    Ok(())
}
