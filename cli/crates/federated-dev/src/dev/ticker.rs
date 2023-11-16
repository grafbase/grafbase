use super::bus::{ComposeMessage, ComposeSender};
use std::time::Duration;

pub(crate) struct Ticker {
    tick_duration: Duration,
    compose_sender: ComposeSender,
}

impl Ticker {
    pub fn new(tick_duration: Duration, compose_sender: ComposeSender) -> Self {
        Self {
            tick_duration,
            compose_sender,
        }
    }

    pub async fn handler(self) -> Result<(), crate::Error> {
        let mut interval = tokio::time::interval(self.tick_duration);

        loop {
            interval.tick().await;

            self.compose_sender.send(ComposeMessage::InitializeRefresh).await?;
        }
    }
}
