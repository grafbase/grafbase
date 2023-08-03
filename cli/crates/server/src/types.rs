use common::types::{LogLevel, UdfKind};
use std::path::PathBuf;

pub const ASSETS_GZIP: &[u8] = include_bytes!("../assets/assets.tar.gz");

#[serde_with::serde_as]
#[derive(serde::Deserialize, Clone, Debug)]
pub enum LogEventType {
    OperationStarted {
        name: Option<String>,
    },
    OperationCompleted {
        name: Option<String>,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
        r#type: common::types::OperationType,
    },
    BadRequest {
        name: Option<String>,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
    },
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
    UdfMessage {
        udf_kind: UdfKind,
        udf_name: String,
        level: LogLevel,
        message: String,
    },
    OperationLogMessage {
        request_id: String,
        event_type: LogEventType,
    },
    CompilationError(String),
}
