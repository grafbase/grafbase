use std::{collections::HashMap, sync::Arc};

use grafbase_telemetry::otel::opentelemetry::trace::TraceId;
use wasmtime::{
    StoreContextMut,
    component::{LinkerInstance, Resource, ResourceType},
};

use crate::{
    names::{
        CONTEXT_DELETE_METHOD, CONTEXT_GET_METHOD, CONTEXT_RESOURCE, CONTEXT_SET_METHOD, SHARED_CONTEXT_GET_METHOD,
        SHARED_CONTEXT_RESOURCE, SHARED_CONTEXT_TRACE_ID_METHOD,
    },
    state::WasiState,
};

/// The internal per-request context storage. Accessible from all hooks throughout a single request
pub type ContextMap = HashMap<String, String>;

/// The internal per-request context storage, read-only.
#[derive(Clone)]
pub struct SharedContext {
    /// Key-value storage.
    kv: Arc<HashMap<String, String>>,
    /// A log channel for access logs.
    trace_id: TraceId,
}

// FIXME: Remove me once hooks & extensions context are merged.
impl Default for SharedContext {
    fn default() -> Self {
        Self {
            kv: Arc::new(HashMap::new()),
            trace_id: TraceId::INVALID,
        }
    }
}

impl SharedContext {
    /// Creates a new shared context.
    pub fn new(kv: Arc<HashMap<String, String>>, trace_id: TraceId) -> Self {
        Self { kv, trace_id }
    }
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
pub(crate) fn inject_mapping(types: &mut LinkerInstance<'_, WasiState>) -> crate::Result<()> {
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
///         trace-id: func() -> string;
///         access-log: func(data: list<u8>) -> result<_, log-error>;
///     }
/// }
/// ```
pub(crate) fn inject_shared_mapping(types: &mut LinkerInstance<'_, WasiState>) -> crate::Result<()> {
    types.resource(
        SHARED_CONTEXT_RESOURCE,
        ResourceType::host::<SharedContext>(),
        |_, _| Ok(()),
    )?;

    types.func_wrap(SHARED_CONTEXT_GET_METHOD, get_shared)?;
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

/// Gives the current opentelemetry trace id.
fn trace_id(store: StoreContextMut<'_, WasiState>, (this,): (Resource<SharedContext>,)) -> anyhow::Result<(String,)> {
    let context = store.data().get(&this).expect("must exist");
    Ok((context.trace_id.to_string(),))
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
