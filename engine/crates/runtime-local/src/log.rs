use runtime::log::{LogEvent, LogEventReceiver, LogEventType};

use crate::bridge::Bridge;

pub struct LogEventReceiverImpl {
    bridge: Bridge,
}

impl LogEventReceiverImpl {
    pub fn new(bridge: Bridge) -> Self {
        Self { bridge }
    }
}

#[async_trait::async_trait]
impl LogEventReceiver for LogEventReceiverImpl {
    async fn invoke<'a>(&self, request_id: &str, _parent_log_event_id: Option<ulid::Ulid>, r#type: LogEventType<'a>) {
        let _ = self
            .bridge
            .request::<_, serde::de::IgnoredAny>("log-event", LogEvent { request_id, r#type })
            .await;
    }
}
