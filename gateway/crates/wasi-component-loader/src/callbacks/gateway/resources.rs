use wasmtime::{
    component::{LinkerInstance, Resource, ResourceType},
    StoreContextMut,
};

use crate::{
    names::{
        GATEWAY_REQUEST_RESOURCE, GET_DOCUMENT_ID_METHOD, GET_OPERATION_NAME_METHOD, SET_DOCUMENT_ID_METHOD,
        SET_OPERATION_NAME_METHOD,
    },
    state::WasiState,
};

/// Maps the needed resources for gateway request manipulation.
///
/// ```ignore
/// interface types {
///     resource gateway-request {
///         get-operation-name: func() -> option<string>;
///         set-operation-name: func(name: option<string>);
///         get-document-id: func() -> option<string>;
///         set-document-id: func(id: option<string>);   
///     }    
/// }
/// ```
pub(crate) fn map(types: &mut LinkerInstance<'_, WasiState>) -> crate::Result<()> {
    types.resource(
        GATEWAY_REQUEST_RESOURCE,
        ResourceType::host::<engine::Request>(),
        |_, _| Ok(()),
    )?;

    types.func_wrap(GET_OPERATION_NAME_METHOD, get_operation_name)?;
    types.func_wrap(SET_OPERATION_NAME_METHOD, set_operation_name)?;

    types.func_wrap(GET_DOCUMENT_ID_METHOD, get_document_id)?;
    types.func_wrap(SET_DOCUMENT_ID_METHOD, set_document_id)?;

    Ok(())
}

/// Gets the name of the operation in the current request
///
/// `get-operation-name: func() -> option<string>`
fn get_operation_name(
    store: StoreContextMut<'_, WasiState>,
    (this,): (Resource<engine::Request>,),
) -> anyhow::Result<(Option<String>,)> {
    let request = store.data().get(&this).expect("must exist");
    let name = request.operation_plan_cache_key.operation_name.clone();

    Ok((name,))
}

/// Sets the name of the operation in the current request
///
/// `set-operation-name: func(name: option<string>)`
fn set_operation_name(
    mut store: StoreContextMut<'_, WasiState>,
    (this, name): (Resource<engine::Request>, Option<String>),
) -> anyhow::Result<()> {
    let request = store.data_mut().get_mut(&this).expect("must exist");
    request.operation_plan_cache_key.operation_name = name;

    Ok(())
}

/// Gets the document id for the current request (trusted docs)
///
/// `get-document-id: func() -> option<string>`
fn get_document_id(
    store: StoreContextMut<'_, WasiState>,
    (this,): (Resource<engine::Request>,),
) -> anyhow::Result<(Option<String>,)> {
    let request = store.data().get(&this).expect("must exist");
    let id = request.operation_plan_cache_key.document_id.clone();

    Ok((id,))
}

/// Sets the document id for the current request (trusted docs)
///
/// `set-document-id: func(id: option<string>)`
fn set_document_id(
    mut store: StoreContextMut<'_, WasiState>,
    (this, id): (Resource<engine::Request>, Option<String>),
) -> anyhow::Result<()> {
    let request = store.data_mut().get_mut(&this).expect("must exist");
    request.operation_plan_cache_key.document_id = id;

    Ok(())
}
