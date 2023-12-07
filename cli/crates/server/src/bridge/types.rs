use common::types::UdfKind;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UdfInvocation {
    pub request_id: String,
    pub name: String,
    pub payload: serde_json::Value,
    pub udf_kind: UdfKind,
}

#[serde_with::serde_as]
#[derive(Deserialize, Debug)]
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
    NestedRequest {
        url: String,
        method: String,
        #[serde(rename = "statusCode")]
        status_code: u16,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
        body: Option<String>,
        #[serde(rename = "contentType")]
        content_type: Option<String>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub request_id: String,
    pub r#type: LogEventType,
}
