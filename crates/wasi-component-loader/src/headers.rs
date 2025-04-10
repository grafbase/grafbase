use std::str::FromStr;

use http::{HeaderName, HeaderValue};
use wasmtime::{
    StoreContextMut,
    component::{ComponentType, LinkerInstance, Lower, Resource, ResourceType},
};

use crate::{
    names::{HEADERS_DELETE_METHOD, HEADERS_ENTRIES_METHOD, HEADERS_GET_METHOD, HEADERS_RESOURCE, HEADERS_SET_METHOD},
    state::{WasiState, WasmOwnedOrBorrowed},
};

pub type HookHeaders = WasmOwnedOrBorrowed<http::HeaderMap>;

#[derive(Debug, ComponentType, Lower, Clone, Copy)]
#[component(enum)]
#[repr(u8)]
enum HeaderError {
    #[component(name = "invalid-header-value")]
    InvalidHeaderValue,
    #[component(name = "invalid-header-name")]
    InvalidHeaderName,
}

/// Map headers resource, with get and set accessors to the guest component.
///
/// ```ignore
/// interface types {
///     resource headers {
///         get: func(key: string) -> result<option<string>, header-error>;
///         set: func(key: string, value: string) -> result<_, header-error>;
///         delete: func(key: string) -> result<option<string>, header-error>;
///         entries: func() -> list<tuple<string, string>>;
///     }
/// }
/// ```
pub(crate) fn inject_mapping(types: &mut LinkerInstance<'_, WasiState>) -> crate::Result<()> {
    types.resource(
        HEADERS_RESOURCE,
        ResourceType::host::<HookHeaders>(),
        |mut store, rep| {
            let resource = wasmtime::component::Resource::<HookHeaders>::new_own(rep);
            // FIXME: Doesn't seem like a great idea, but today if it's an owned version it means
            // the caller provided it in the store and will cleanup himself afterwards...
            if store.data_mut().get(&resource).unwrap().is_guest_borrowed() {
                store
                    .data_mut()
                    .table
                    .delete(wasmtime::component::Resource::<HookHeaders>::new_own(rep))?;
            }
            Ok(())
        },
    )?;
    types.func_wrap(HEADERS_SET_METHOD, set)?;
    types.func_wrap(HEADERS_GET_METHOD, get)?;
    types.func_wrap(HEADERS_DELETE_METHOD, delete)?;
    types.func_wrap(HEADERS_ENTRIES_METHOD, entries)?;

    Ok(())
}

/// Modify or add a header with the given key and value. Will return an error to the user
/// if the key or value contains a non-ascii character.
///
/// `set: func(key: string, value: string) -> result<_, header-error>`
fn set(
    mut store: StoreContextMut<'_, WasiState>,
    (this, key, value): (Resource<HookHeaders>, String, String),
) -> anyhow::Result<(Result<(), HeaderError>,)> {
    let headers = store.data_mut().get_ref_mut(&this).expect("must exist");

    let key = match HeaderName::from_str(&key) {
        Ok(key) => key,
        Err(_) => return Ok((Err(HeaderError::InvalidHeaderName),)),
    };

    let value = match HeaderValue::from_str(&value) {
        Ok(value) => value,
        Err(_) => return Ok((Err(HeaderError::InvalidHeaderValue),)),
    };

    headers.insert(key, value);

    Ok((Ok(()),))
}

/// Look for a header with the given key, returning a copy of the value if found. Will return an
/// error to the user if the key contains a non-ascii character.
///
/// `get: func(key: string) -> result<option<string>, header-error>`
fn get(
    mut store: StoreContextMut<'_, WasiState>,
    (this, key): (Resource<HookHeaders>, String),
) -> anyhow::Result<(Option<String>,)> {
    let headers = store.data_mut().get_ref(&this).expect("must exist");

    let value = headers
        .get(&key)
        .map(|val| String::from_utf8_lossy(val.as_bytes()).into_owned());

    Ok((value,))
}

/// Look for a header with the given key, returning a copy of the value if found. Will remove
/// the value from the headers.
///
/// `delete: func(key: string) -> result<option<string>, header-error>`
fn delete(
    mut store: StoreContextMut<'_, WasiState>,
    (this, key): (Resource<HookHeaders>, String),
) -> anyhow::Result<(Option<String>,)> {
    let headers = store.data_mut().get_ref_mut(&this).expect("must exist");

    let old_value = headers
        .remove(&key)
        .map(|val| String::from_utf8_lossy(val.as_bytes()).into_owned());

    Ok((old_value,))
}

fn entries(
    mut store: StoreContextMut<'_, WasiState>,
    (this,): (Resource<HookHeaders>,),
) -> anyhow::Result<(Vec<(String, String)>,)> {
    let headers = store.data_mut().get_ref_mut(&this).expect("must exist");

    let entries = headers
        .iter()
        .map(|(key, value)| {
            let key = key.to_string();
            let value = String::from_utf8_lossy(value.as_bytes()).into_owned();
            (key, value)
        })
        .collect();

    Ok((entries,))
}
