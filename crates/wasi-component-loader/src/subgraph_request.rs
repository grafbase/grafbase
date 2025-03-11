use runtime::hooks::SubgraphRequest;
use wasmtime::{
    StoreContextMut,
    component::{LinkerInstance, Resource, ResourceType},
};

use crate::{
    extension::api::wit::grafbase::sdk::http_client::HttpMethod,
    headers::Headers,
    names::{
        SUBGRAPH_REQUEST_GET_HEADERS_METHOD, SUBGRAPH_REQUEST_GET_METHOD_METHOD, SUBGRAPH_REQUEST_GET_URL_METHOD,
        SUBGRAPH_REQUEST_RESOURCE, SUBGRAPH_REQUEST_SET_METHOD_METHOD, SUBGRAPH_REQUEST_SET_URL_METHOD,
    },
    state::WasiState,
};

pub(crate) fn inject_mapping(types: &mut LinkerInstance<'_, WasiState>) -> crate::Result<()> {
    types.resource(
        SUBGRAPH_REQUEST_RESOURCE,
        ResourceType::host::<SubgraphRequest>(),
        |_, _| Ok(()),
    )?;
    types.func_wrap(SUBGRAPH_REQUEST_GET_METHOD_METHOD, get_method)?;
    types.func_wrap(SUBGRAPH_REQUEST_SET_METHOD_METHOD, set_method)?;
    types.func_wrap(SUBGRAPH_REQUEST_GET_URL_METHOD, get_url)?;
    types.func_wrap(SUBGRAPH_REQUEST_SET_URL_METHOD, set_url)?;
    types.func_wrap(SUBGRAPH_REQUEST_GET_HEADERS_METHOD, get_headers)?;

    Ok(())
}

fn get_method(
    store: StoreContextMut<'_, WasiState>,
    (this,): (Resource<SubgraphRequest>,),
) -> anyhow::Result<(HttpMethod,)> {
    let request = store.data().get(&this).expect("must exist");
    Ok((request.method.clone().into(),))
}

fn set_method(
    mut store: StoreContextMut<'_, WasiState>,
    (this, method): (Resource<SubgraphRequest>, HttpMethod),
) -> anyhow::Result<()> {
    let request = store.data_mut().get_mut(&this).expect("must exist");
    request.method = method.into();
    Ok(())
}

fn set_url(
    mut store: StoreContextMut<'_, WasiState>,
    (this, url): (Resource<SubgraphRequest>, String),
) -> anyhow::Result<(Result<(), String>,)> {
    let request = store.data_mut().get_mut(&this).expect("must exist");

    match url.parse::<url::Url>() {
        Ok(url) => {
            request.url = url;
            Ok((Ok(()),))
        }
        Err(err) => Ok((Err(err.to_string()),)),
    }
}

fn get_url(store: StoreContextMut<'_, WasiState>, (this,): (Resource<SubgraphRequest>,)) -> anyhow::Result<(String,)> {
    let request = store.data().get(&this).expect("must exist");

    Ok((request.url.to_string(),))
}

fn get_headers(
    mut store: StoreContextMut<'_, WasiState>,
    (this,): (Resource<SubgraphRequest>,),
) -> anyhow::Result<(Resource<Headers>,)> {
    let headers = crate::state::get_child_ref!(store, this: SubgraphRequest => headers: http::HeaderMap);
    Ok((headers,))
}
