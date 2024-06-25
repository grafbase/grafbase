use grafbase_tracing::span::GRAFBASE_TARGET;
use http::HeaderMap;
use wasmtime::{
    component::{Instance, Resource, TypedFunc},
    Store,
};

use crate::{
    names::{COMPONENT_GATEWAY_REQUEST, GATEWAY_HOOK_FUNCTION},
    state::WasiState,
    ComponentLoader, ContextMap, ErrorResponse,
};

/// The hook function takes two parameters: the context and the headers.
/// They are wrapped as resources, meaning they are in a shared memory space
/// accessible from the host and from the guest.
pub(crate) type Parameters = (Resource<ContextMap>, Resource<HeaderMap>);

/// The guest can read and modify the input headers and request as it wishes. A successful
/// call returns unit. The user can return an error response, which should be mapped to a
/// corresponding HTTP status code.
pub(crate) type Response = (Result<(), ErrorResponse>,);

/// The gateway hook is called right after authentication.
///
/// An instance of a function to be called from the Gateway level for the request.
/// The instance is meant to be separate for every request. The instance shares a memory space
/// with the guest, and cannot be shared with multiple requests.
pub struct GatewayHookInstance {
    store: Store<WasiState>,
    headers: Resource<HeaderMap>,
    context: Resource<ContextMap>,
    hook: Option<TypedFunc<Parameters, Response>>,
}

impl GatewayHookInstance {
    pub(crate) async fn new(loader: &ComponentLoader, context: ContextMap, headers: HeaderMap) -> crate::Result<Self> {
        let mut store = super::initialize_store(loader.config(), loader.engine())?;

        // adds the data to the shared memory
        let context = store.data_mut().push_resource(context)?;
        let headers = store.data_mut().push_resource(headers)?;

        let instance = loader
            .linker()
            .instantiate_async(&mut store, loader.component())
            .await?;

        let hook = get_hook(&mut store, &instance);

        Ok(Self {
            store,
            headers,
            context,
            hook,
        })
    }

    pub(crate) async fn call(mut self) -> crate::Result<(ContextMap, HeaderMap)> {
        // we need to take the pointers now, because a resource is not Copy and we need
        // the pointers to get the data back from the shared memory.
        let headers_rep = self.headers.rep();
        let context_rep = self.context.rep();

        if let Some(hook) = self.hook {
            hook.call_async(&mut self.store, (self.context, self.headers))
                .await?
                .0?;
        };

        // take the data back from the shared memory
        let context = self.store.data_mut().take_resource(context_rep)?;
        let headers = self.store.data_mut().take_resource(headers_rep)?;

        Ok((context, headers))
    }
}

fn get_hook(store: &mut Store<WasiState>, instance: &Instance) -> Option<TypedFunc<Parameters, Response>> {
    let mut exports = instance.exports(store);
    let mut root = exports.root();

    let Some(mut interface) = root.instance(COMPONENT_GATEWAY_REQUEST) else {
        tracing::debug!(target: GRAFBASE_TARGET, "could not find export for gateway-request interface");
        return None;
    };

    match interface.typed_func(GATEWAY_HOOK_FUNCTION) {
        Ok(hook) => {
            tracing::debug!(target: GRAFBASE_TARGET, "instantized the gateway hook WASM function");
            Some(hook)
        }
        // the user has not defined the hook function, so we return none and do not try to call this function.
        Err(e) => {
            tracing::debug!(target: GRAFBASE_TARGET, "error instantizing the gateway hook WASM function: {e}");
            None
        }
    }
}
