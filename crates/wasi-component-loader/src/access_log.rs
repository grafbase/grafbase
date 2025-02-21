use crossbeam::{channel::TrySendError, sync::WaitGroup};
use grafbase_telemetry::otel::opentelemetry::metrics::UpDownCounter;
use wasmtime::{
    StoreContextMut,
    component::{ComponentType, LinkerInstance, Lower, ResourceType},
};

use crate::{
    names::{ACCESS_LOG_RESOURCE, ACCESS_LOG_SEND_FUNCTION},
    state::WasiState,
};

/// Sender for a wasi hook to send logs to the writer.
#[derive(Clone)]
pub struct ChannelLogSender {
    sender: crossbeam::channel::Sender<AccessLogMessage>,
    lossy_log: bool,
    pending_logs_counter: UpDownCounter<i64>,
}

impl ChannelLogSender {
    /// Sends the given access log message to the access log.
    pub fn send(&self, data: AccessLogMessage) -> Result<(), LogError> {
        if self.lossy_log {
            match self.sender.try_send(data) {
                Ok(_) => (),
                Err(TrySendError::Full(AccessLogMessage::Data(data))) => return Err(LogError::ChannelFull(data)),
                Err(_) => return Err(LogError::ChannelClosed),
            }
        } else if self.sender.send(data).is_err() {
            return Err(LogError::ChannelClosed);
        }

        self.pending_logs_counter.add(1, &[]);

        Ok(())
    }

    /// Wait until all access logs are written to the file.
    pub async fn graceful_shutdown(&self) {
        let wg = WaitGroup::new();

        if self.sender.send(AccessLogMessage::Shutdown(wg.clone())).is_err() {
            tracing::debug!("access log receiver is already dead, cannot empty log channel");
        }

        tokio::task::spawn_blocking(|| wg.wait()).await.unwrap();
    }
}

/// A receiver for the logger to receive messages and write them somewhere.
pub type ChannelLogReceiver = crossbeam::channel::Receiver<AccessLogMessage>;

#[derive(Debug, ComponentType, Lower)]
#[component(variant)]
pub enum LogError {
    #[component(name = "channel-full")]
    ChannelFull(Vec<u8>),
    #[component(name = "channel-closed")]
    ChannelClosed,
}

/// https://github.com/tokio-rs/tracing/blob/master/tracing-appender/src/non_blocking.rs#L61-L70
const DEFAULT_BUFFERED_LINES_LIMIT: usize = 128_000;

/// Creates a new channel for access logs.
pub fn create_log_channel(
    lossy_log: bool,
    pending_logs_counter: UpDownCounter<i64>,
) -> (ChannelLogSender, ChannelLogReceiver) {
    let (sender, receiver) = crossbeam::channel::bounded(DEFAULT_BUFFERED_LINES_LIMIT);

    (
        ChannelLogSender {
            sender,
            lossy_log,
            pending_logs_counter,
        },
        receiver,
    )
}

/// A message sent through access log channel.
pub enum AccessLogMessage {
    /// Write data to the logs.
    Data(Vec<u8>),
    /// Shutdown the channel.
    Shutdown(WaitGroup),
}

impl AccessLogMessage {
    /// Convert the message into data bytes, if present.
    pub fn into_data(self) -> Option<Vec<u8>> {
        match self {
            AccessLogMessage::Data(data) => Some(data),
            AccessLogMessage::Shutdown(_) => None,
        }
    }
}

pub(crate) fn inject_mapping(types: &mut LinkerInstance<'_, WasiState>) -> crate::Result<()> {
    types.resource(ACCESS_LOG_RESOURCE, ResourceType::host::<()>(), |_, _| Ok(()))?;
    types.func_wrap(ACCESS_LOG_SEND_FUNCTION, access_log_send)?;

    Ok(())
}

fn access_log_send(
    ctx: StoreContextMut<'_, WasiState>,
    (data,): (Vec<u8>,),
) -> anyhow::Result<(Result<(), LogError>,)> {
    let sender = ctx.data().access_log();

    let data = AccessLogMessage::Data(data);

    let Err(e) = sender.send(data) else {
        return Ok((Ok(()),));
    };

    match e {
        LogError::ChannelFull(_) => {
            tracing::error!("access log channel is over capacity");
        }
        LogError::ChannelClosed => {
            tracing::error!("access log channel closed");
        }
    }

    Ok((Err(e),))
}
