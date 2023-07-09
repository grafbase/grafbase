use common::types::{UdfKind, UdfMessageLevel};
use std::path::PathBuf;

pub const ASSETS_GZIP: &[u8] = include_bytes!("../assets/assets.tar.gz");

#[derive(Clone, Debug)]
pub enum ServerMessage {
    Ready(u16),
    Reload(PathBuf),
    InstallUdfDependencies,
    CompleteInstallingUdfDependencies {
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
    UdfMessage {
        udf_kind: UdfKind,
        udf_name: String,
        level: UdfMessageLevel,
        message: String,
    },
    OperationStarted {
        request_id: String,
        name: Option<String>,
    },
    OperationCompleted {
        request_id: String,
        name: Option<String>,
        duration: std::time::Duration,
    },
    CompilationError(String),
}
