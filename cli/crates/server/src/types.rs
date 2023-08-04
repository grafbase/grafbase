use common::types::{LogLevel, UdfKind};
use std::path::PathBuf;

pub const ASSETS_GZIP: &[u8] = include_bytes!("../assets/assets.tar.gz");

#[derive(Clone, Copy, Debug)]
pub enum RequestCompletedOutcome {
    Success { r#type: common::types::OperationType },
    BadRequest,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub enum NestedRequestScopedMessage {
    UdfMessage {
        udf_kind: UdfKind,
        udf_name: String,
        level: LogLevel,
        message: String,
    },
}

#[derive(Clone, Debug)]
pub enum LogEventType {
    RequestCompleted {
        name: Option<String>,
        duration: std::time::Duration,
        request_completed_type: RequestCompletedOutcome,
    },
    NestedEvent(NestedRequestScopedMessage),
}

#[derive(Clone, Debug)]
pub enum ServerMessage {
    Ready(u16),
    Reload(PathBuf),
    StartUdfBuild {
        udf_kind: UdfKind,
        udf_name: String,
    },
    CompleteUdfBuild {
        udf_kind: UdfKind,
        udf_name: String,
        duration: std::time::Duration,
    },
    RequestScopedMessage {
        request_id: String,
        event_type: LogEventType,
    },
    CompilationError(String),
}
