pub(crate) mod resources;

use grafbase_tracing::span::GRAFBASE_TARGET;
use http::HeaderMap;
use wasmtime::{
    component::{Resource, TypedFunc},
    Store,
};

use crate::{names::GATEWAY_CALLBACK_FUNCTION, state::WasiState, ComponentLoader, ErrorResponse};

/// The callback function takes two parameters: the headers and the request.
/// They are wrapped as resources, meaning they are in a shared memory space
/// accessible from the host and from the guest.
pub(crate) type CallbackParameters = (Resource<HeaderMap>, Resource<engine::Request>);

/// The guest can read and modify the input headers and request as it wishes. A successful
/// call returns unit. The user can return an error response, which should be mapped to a
/// corresponding HTTP status code.
pub(crate) type CallbackResponse = (Result<(), ErrorResponse>,);

/// An instance of a function to be called from the Gateway level for the request.
/// The instance is meant to be separate for every request. The instance shares a memory space
/// with the guest, and cannot be shared with multiple requests.
pub struct GatewayCallbackInstance {
    store: Store<WasiState>,
    headers: Resource<HeaderMap>,
    request: Resource<engine::Request>,
    callback: Option<TypedFunc<CallbackParameters, CallbackResponse>>,
}

impl GatewayCallbackInstance {
    pub(crate) async fn new(
        loader: &ComponentLoader,
        headers: HeaderMap,
        request: engine::Request,
    ) -> crate::Result<Self> {
        let mut store = super::initialize_store(loader.config(), loader.engine())?;

        // adds the data to the shared memory
        let headers = store.data_mut().push_resource(headers)?;
        let request = store.data_mut().push_resource(request)?;

        let instance = loader
            .linker()
            .instantiate_async(&mut store, loader.component())
            .await?;

        let callback = match instance.get_typed_func(&mut store, GATEWAY_CALLBACK_FUNCTION) {
            Ok(callback) => {
                tracing::debug!(target: GRAFBASE_TARGET, "instantized the gateway callback WASM function");

                Some(callback)
            }
            // the user has not defined the callback function, so we return none and do not try to call this function.
            Err(e) => {
                tracing::debug!(target: GRAFBASE_TARGET, "error instantizing the gateway callback WASM function: {e}");

                None
            }
        };

        Ok(Self {
            store,
            headers,
            request,
            callback,
        })
    }

    pub(crate) async fn call(mut self) -> crate::Result<(HeaderMap, engine::Request)> {
        // we need to take the pointers now, because a resource is not Copy and we need
        // the pointers to get the data back from the shared memory.
        let headers_rep = self.headers.rep();
        let request_rep = self.request.rep();

        if let Some(callback) = self.callback {
            callback
                .call_async(&mut self.store, (self.headers, self.request))
                .await?
                .0?;
        };

        // take the data back from the shared memory
        let headers = self.store.data_mut().take_resource(headers_rep)?;
        let request = self.store.data_mut().take_resource(request_rep)?;

        Ok((headers, request))
    }
}
