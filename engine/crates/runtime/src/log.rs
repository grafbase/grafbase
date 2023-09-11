pub use grafbase_types::LogEventType;

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent<'a> {
    pub request_id: &'a str,
    pub r#type: LogEventType<'a>,
}

#[async_trait::async_trait]
pub trait LogEventReceiver {
    async fn invoke<'a>(&self, request_id: &str, request: LogEventType<'a>);
}
