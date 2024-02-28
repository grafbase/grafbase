use common::types::{LogLevel, UdfKind};
use std::{net::SocketAddr, path::PathBuf};

pub const ASSETS_GZIP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/assets.tar.gz"));

#[derive(Clone, Copy, Debug)]
pub enum RequestCompletedOutcome {
    Success { r#type: common::types::OperationType },
    BadRequest,
}

#[derive(Clone, Debug)]
pub enum NestedRequestScopedMessage {
    UdfMessage {
        udf_kind: UdfKind,
        udf_name: String,
        level: LogLevel,
        message: String,
    },
    NestedRequest {
        url: String,
        method: String,
        status_code: u16,
        duration: std::time::Duration,
        body: Option<String>,
        content_type: Option<String>,
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

pub type MessageSender = tokio::sync::mpsc::UnboundedSender<ServerMessage>;

#[derive(Clone, Debug)]
pub enum ServerMessage {
    Ready {
        listen_address: SocketAddr,
        is_federated: bool,
    },
    Reload(PathBuf),
    StartUdfBuildAll,
    CompleteUdfBuildAll {
        duration: std::time::Duration,
    },
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
