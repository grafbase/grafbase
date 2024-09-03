use std::{collections::HashMap, sync::Arc};

use crossbeam::{channel::TrySendError, sync::WaitGroup};
use opentelemetry::trace::TraceContextExt;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use wasmtime::{
    component::{ComponentType, LinkerInstance, Lower, Resource, ResourceType},
    StoreContextMut,
};

use crate::{
    names::{
        CONTEXT_DELETE_METHOD, CONTEXT_GET_METHOD, CONTEXT_RESOURCE, CONTEXT_SET_METHOD,
        SHARED_CONTEXT_ACCESS_LOG_METHOD, SHARED_CONTEXT_GET_METHOD, SHARED_CONTEXT_RESOURCE,
        SHARED_CONTEXT_TRACE_ID_METHOD,
    },
    state::WasiState,
};

/// Sender for a wasi hook to send logs to the writer.
#[derive(Clone)]
pub struct ChannelLogSender {
    sender: crossbeam::channel::Sender<AccessLogMessage>,
    lossy_log: bool,
}

impl ChannelLogSender {
    /// Sends the given access log message to the access log.
    pub fn send(&self, data: AccessLogMessage) -> Result<(), LogError> {
        if self.lossy_log {
            if let Err(e) = self.sender.try_send(data) {
                match e {
                    TrySendError::Full(AccessLogMessage::Data(data)) => return Err(LogError::ChannelFull(data)),
                    _ => return Err(LogError::ChannelClosed),
                }
            }
        } else if self.sender.send(data).is_err() {
            return Err(LogError::ChannelClosed);
        }

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

/// https://github.com/tokio-rs/tracing/blob/master/tracing-appender/src/non_blocking.rs#L61-L70
const DEFAULT_BUFFERED_LINES_LIMIT: usize = 128_000;

/// Creates a new channel for access logs.
pub fn create_log_channel(lossy_log: bool) -> (ChannelLogSender, ChannelLogReceiver) {
    let (sender, receiver) = crossbeam::channel::bounded(DEFAULT_BUFFERED_LINES_LIMIT);
    (ChannelLogSender { sender, lossy_log }, receiver)
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

/// The internal per-request context storage. Accessible from all hooks throughout a single request
pub type ContextMap = HashMap<String, String>;

/// The internal per-request context storage, read-only.
#[derive(Clone)]
pub struct SharedContext {
    /// Key-value storage.
    kv: Arc<HashMap<String, String>>,
    /// A log channel for access logs.
    access_log: ChannelLogSender,
    span: Span,
}

impl SharedContext {
    /// Creates a new shared context.
    pub fn new(kv: Arc<HashMap<String, String>>, access_log: ChannelLogSender, span: Span) -> Self {
        Self { kv, access_log, span }
    }
}

#[derive(Debug, ComponentType, Lower)]
#[component(variant)]
pub enum LogError {
    #[component(name = "channel-full")]
    ChannelFull(Vec<u8>),
    #[component(name = "channel-closed")]
    ChannelClosed,
}

/// Map context resource, with get and set accessors to the guest component.
///
/// ```ignore
/// interface types {
///     resource context {
///         get: func(key: string) -> option<string>;
///         set: func(key: string, value: string);
///         delete: func(key: string) -> option<string>;
///     }    
/// }
/// ```
pub(crate) fn map(types: &mut LinkerInstance<'_, WasiState>) -> crate::Result<()> {
    types.resource(CONTEXT_RESOURCE, ResourceType::host::<ContextMap>(), |_, _| Ok(()))?;
    types.func_wrap(CONTEXT_SET_METHOD, set)?;
    types.func_wrap(CONTEXT_GET_METHOD, get)?;
    types.func_wrap(CONTEXT_DELETE_METHOD, delete)?;

    Ok(())
}

/// Map read-only context resource, with only the get accessor.
///
/// ```ignore
/// interface types {
///     resource shared-context {
///         get: func(key: string) -> option<string>;
///         access-log: func(data: list<u8>) -> result<_, log-error>;
///     }
/// }
/// ```
pub(crate) fn map_shared(types: &mut LinkerInstance<'_, WasiState>) -> crate::Result<()> {
    types.resource(
        SHARED_CONTEXT_RESOURCE,
        ResourceType::host::<SharedContext>(),
        |_, _| Ok(()),
    )?;

    types.func_wrap(SHARED_CONTEXT_GET_METHOD, get_shared)?;
    types.func_wrap(SHARED_CONTEXT_ACCESS_LOG_METHOD, log_access)?;
    types.func_wrap(SHARED_CONTEXT_TRACE_ID_METHOD, trace_id)?;

    Ok(())
}

/// Modify or add to the context wwith the given key and value.
///
/// `set: func(key: string, value: string)`
fn set(
    mut store: StoreContextMut<'_, WasiState>,
    (this, key, value): (Resource<ContextMap>, String, String),
) -> anyhow::Result<()> {
    let context = store.data_mut().get_mut(&this).expect("must exist");
    context.insert(key, value);

    Ok(())
}

/// Look for a context value with the given key, returning a copy of the value if found.
///
/// `get: func(key: string) -> option<string>`
fn get(
    store: StoreContextMut<'_, WasiState>,
    (this, key): (Resource<ContextMap>, String),
) -> anyhow::Result<(Option<String>,)> {
    let context = store.data().get(&this).expect("must exist");
    let val = context.get(&key).cloned();

    Ok((val,))
}

/// Look for a context value with the given key, returning a copy of the value if found.
///
/// `get: func(key: string) -> option<string>`
fn get_shared(
    store: StoreContextMut<'_, WasiState>,
    (this, key): (Resource<SharedContext>, String),
) -> anyhow::Result<(Option<String>,)> {
    let context = store.data().get(&this).expect("must exist");
    let val = context.kv.get(&key).cloned();

    Ok((val,))
}

/// Sends data to the access log channel.
fn log_access(
    store: StoreContextMut<'_, WasiState>,
    (this, data): (Resource<SharedContext>, Vec<u8>),
) -> anyhow::Result<(Result<(), LogError>,)> {
    let context = store.data().get(&this).expect("must exist");
    let data = AccessLogMessage::Data(data);

    match context.access_log.send(data) {
        Ok(()) => Ok((Ok(()),)),
        Err(e) => {
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
    }
}

/// Gives the current opentelemetry trace id.
fn trace_id(store: StoreContextMut<'_, WasiState>, (this,): (Resource<SharedContext>,)) -> anyhow::Result<(String,)> {
    let context = store.data().get(&this).expect("must exist");
    let trace_id = context.span.context().span().span_context().trace_id();

    Ok((trace_id.to_string(),))
}

/// Look for a context value with the given key, returning a copy of the value if found. Will remove
/// the value from the headers.
///
/// `delete: func(key: string) -> result<option<string>, header-error>`
fn delete(
    mut store: StoreContextMut<'_, WasiState>,
    (this, key): (Resource<ContextMap>, String),
) -> anyhow::Result<(Option<String>,)> {
    let context = store.data_mut().get_mut(&this).expect("must exist");
    let val = context.remove(&key);

    Ok((val,))
}
