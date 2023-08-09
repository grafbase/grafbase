#[derive(serde::Serialize, Debug)]
pub enum OperationType {
    Query { is_introspection: bool },
    Mutation,
    Subscription,
}

#[serde_with::serde_as]
#[derive(serde::Serialize, Debug)]
pub enum LogEventType<'a> {
    OperationStarted {
        name: Option<&'a str>,
    },
    OperationCompleted {
        name: Option<&'a str>,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
        r#type: OperationType,
    },
    BadRequest {
        name: Option<&'a str>,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
    },
}

#[derive(serde::Serialize, Debug)]
pub struct LogEvent<'a> {
    pub request_id: &'a str,
    pub r#type: LogEventType<'a>,
}

#[async_trait::async_trait]
pub trait LogEventReceiver {
    async fn invoke<'a>(&self, request_id: &str, request: LogEventType<'a>);
}
