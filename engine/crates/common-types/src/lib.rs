pub mod auth;

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
)]
#[strum(serialize_all = "snake_case")]
pub enum UdfKind {
    Resolver,
    Authorizer,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug)]
pub enum OperationType {
    Query { is_introspection: bool },
    Mutation,
    Subscription,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

#[serde_with::serde_as]
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum LogEventType<'a> {
    OperationStarted {
        name: Option<std::borrow::Cow<'a, str>>,
    },
    OperationCompleted {
        name: Option<std::borrow::Cow<'a, str>>,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
        r#type: OperationType,
    },
    BadRequest {
        name: Option<std::borrow::Cow<'a, str>>,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
    },
    GatewayRequest {
        url: String,
        method: String,
        #[serde(rename = "statusCode")]
        status_code: u16,
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
    UdfMessage {
        level: LogLevel,
        message: String,
    },
}

impl LogEventType<'_> {
    pub fn log_level(&self) -> LogLevel {
        match self {
            LogEventType::OperationStarted { .. }
            | LogEventType::OperationCompleted { .. }
            | LogEventType::NestedRequest { .. } => LogLevel::Info,
            LogEventType::GatewayRequest { status_code, .. } => match *status_code {
                200..=299 => LogLevel::Info,
                _other => LogLevel::Error,
            },
            LogEventType::BadRequest { .. } => LogLevel::Error,
            LogEventType::UdfMessage { level, .. } => *level,
        }
    }
}
