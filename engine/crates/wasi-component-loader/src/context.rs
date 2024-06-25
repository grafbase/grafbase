use std::{collections::HashMap, sync::Arc};

use wasmtime::{
    component::{LinkerInstance, Resource, ResourceType},
    StoreContextMut,
};

use crate::{
    names::{
        CONTEXT_DELETE_METHOD, CONTEXT_GET_METHOD, CONTEXT_RESOURCE, CONTEXT_SET_METHOD, SHARED_CONTEXT_GET_METHOD,
        SHARED_CONTEXT_RESOURCE,
    },
    state::WasiState,
};

/// The internal per-request context storage. Accessible from all hooks throughout a single request
pub type ContextMap = HashMap<String, String>;

/// The internal per-request context storage, read-only.
pub type SharedContextMap = Arc<HashMap<String, String>>;

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
        ResourceType::host::<SharedContextMap>(),
        |_, _| Ok(()),
    )?;

    types.func_wrap(SHARED_CONTEXT_GET_METHOD, get_shared)?;

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
    (this, key): (Resource<SharedContextMap>, String),
) -> anyhow::Result<(Option<String>,)> {
    let context = store.data().get(&this).expect("must exist");
    let val = context.get(&key).cloned();

    Ok((val,))
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
