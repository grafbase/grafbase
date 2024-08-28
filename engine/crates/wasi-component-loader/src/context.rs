use std::{collections::HashMap, sync::Arc};

use crossbeam::{channel::TrySendError, sync::WaitGroup};
use grafbase_telemetry::span::GRAFBASE_TARGET;
use wasmtime::{
    component::{ComponentType, LinkerInstance, Lower, Resource, ResourceType},
    StoreContextMut,
};

use crate::{
    names::{
        CONTEXT_DELETE_METHOD, CONTEXT_GET_METHOD, CONTEXT_RESOURCE, CONTEXT_SET_METHOD,
        SHARED_CONTEXT_ACCESS_LOG_METHOD, SHARED_CONTEXT_GET_METHOD, SHARED_CONTEXT_RESOURCE,
    },
    state::WasiState,
};

/// Sender for a wasi hook to send logs to the writer.
pub type ChannelLogSender = crossbeam::channel::Sender<AccessLogMessage>;

/// A receiver for the logger to receive messages and write them somewhere.
pub type ChannelLogReceiver = crossbeam::channel::Receiver<AccessLogMessage>;

/// https://github.com/tokio-rs/tracing/blob/master/tracing-appender/src/non_blocking.rs#L61-L70
const DEFAULT_BUFFERED_LINES_LIMIT: usize = 128_000;

/// Creates a new channel for access logs.
pub fn create_log_channel() -> (ChannelLogSender, ChannelLogReceiver) {
    crossbeam::channel::bounded(DEFAULT_BUFFERED_LINES_LIMIT)
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
    /// If true, messages get dropped when the channel is full.
    lossy_log: bool,
}

impl SharedContext {
    /// Creates a new shared context.
    pub fn new(kv: Arc<HashMap<String, String>>, access_log: ChannelLogSender, lossy_log: bool) -> Self {
        Self {
            kv,
            access_log,
            lossy_log,
        }
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

    if context.lossy_log {
        if let Err(e) = context.access_log.try_send(data) {
            match e {
                TrySendError::Full(AccessLogMessage::Data(data)) => {
                    tracing::error!(target: GRAFBASE_TARGET, "access log channel is over capacity");
                    return Ok((Err(LogError::ChannelFull(data)),));
                }
                _ => {
                    tracing::error!(target: GRAFBASE_TARGET, "access log channel closed");
                    return Ok((Err(LogError::ChannelClosed),));
                }
            }
        }
    } else if context.access_log.send(data).is_err() {
        return Ok((Err(LogError::ChannelClosed),));
    }

    Ok((Ok(()),))
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
