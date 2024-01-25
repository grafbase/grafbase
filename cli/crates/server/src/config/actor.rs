use common::channels::constant_watch_receiver;
use futures_util::Stream;
use tokio::sync::watch;
use tokio_stream::{wrappers::WatchStream, StreamExt};

use crate::{
    file_watcher::ChangeStream,
    types::{MessageSender, ServerMessage},
};

use super::{Config, ConfigError, ConfigStream};

pub struct ConfigActor {
    receiver: watch::Receiver<Result<Config, ConfigError>>,
}

impl ConfigActor {
    pub async fn new(files: Option<ChangeStream>, message_sender: MessageSender) -> Self {
        let variables = crate::environment::variables().collect();
        let initial_value = super::build_config(&variables, None).await;

        let Some(mut files) = files else {
            // If we don't have a watcher stream then we're not in watch mode
            // just return a constant receiver with the initial value.
            return ConfigActor {
                receiver: constant_watch_receiver(initial_value),
            };
        };

        let (sender, receiver) = watch::channel(initial_value);

        tokio::spawn(async move {
            while let Some(next) = files.next().await {
                message_sender.send(ServerMessage::Reload(next.clone())).ok();

                let next_result = super::build_config(&variables, Some(next)).await;

                if sender.send(next_result).is_err() {
                    // Channel is closed, so shut down
                    tracing::info!("config watcher shuttiing down");
                    return;
                }
            }
        });

        ConfigActor { receiver }
    }

    pub fn current_result(&self) -> Result<Config, ConfigError> {
        self.receiver.borrow().clone()
    }

    /// A future that resolves when the config next changes
    pub async fn changed(&mut self) -> Result<(), watch::error::RecvError> {
        self.receiver.changed().await
    }

    /// A stream of the config results, including the current value.
    pub fn result_stream(&self) -> impl Stream<Item = Result<Config, ConfigError>> {
        WatchStream::new(self.receiver.clone())
    }

    pub fn config_stream(&self) -> ConfigStream {
        Box::pin(WatchStream::new(self.receiver.clone()).filter_map(|result| result.ok()))
    }

    pub fn into_federated_config_receiver(mut self) -> federated_dev::ConfigWatcher {
        let initial_value = match self.receiver.borrow().as_ref() {
            Ok(config) => config.federated_graph_config.clone().unwrap_or_default(),
            Err(_) => Default::default(),
        };
        let (result_sender, result_receiver) = watch::channel(initial_value);

        tokio::spawn(async move {
            loop {
                if self.receiver.changed().await.is_err() {
                    return;
                }

                let Ok(Config {
                    federated_graph_config: Some(config),
                    ..
                }) = self.receiver.borrow().clone()
                else {
                    continue;
                };

                if result_sender.send(config).is_err() {
                    return;
                }
            }
        });

        result_receiver
    }
}
