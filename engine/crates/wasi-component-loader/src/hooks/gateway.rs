use core::fmt;

use anyhow::anyhow;
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
    hook: Option<TypedFunc<Parameters, Response>>,
    poisoned: bool,
}

impl fmt::Debug for GatewayHookInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "GatewayHookInstance { ... }".fmt(f)
    }
}

impl GatewayHookInstance {
    /// Creates a new instance for the gateway hook.
    pub async fn new(loader: &ComponentLoader) -> crate::Result<Self> {
        let mut store = super::initialize_store(loader.config(), loader.engine())?;

        let instance = loader
            .linker()
            .instantiate_async(&mut store, loader.component())
            .await?;

        let hook = get_hook(&mut store, &instance);

        Ok(Self {
            store,
            hook,
            poisoned: false,
        })
    }

    /// Calls the hook with the given parameters.
    pub async fn call(&mut self, context: ContextMap, headers: HeaderMap) -> crate::Result<(ContextMap, HeaderMap)> {
        match self.hook {
            Some(ref hook) => {
                // adds the data to the shared memory
                let context = self.store.data_mut().push_resource(context)?;
                let headers = self.store.data_mut().push_resource(headers)?;

                // we need to take the pointers now, because a resource is not Copy and we need
                // the pointers to get the data back from the shared memory.
                let headers_rep = headers.rep();
                let context_rep = context.rep();

                let result = hook.call_async(&mut self.store, (context, headers)).await;

                if result.is_err() {
                    self.poisoned = true;
                } else {
                    hook.post_return_async(&mut self.store).await?;
                }

                result?.0?;

                // take the data back from the shared memory
                let context = self.store.data_mut().take_resource(context_rep)?;
                let headers = self.store.data_mut().take_resource(headers_rep)?;

                Ok((context, headers))
            }
            None => Ok((context, headers)),
        }
    }

    /// Resets the store to the original state. This must be called if wanting to reuse this instance.
    ///
    /// If the cleanup fails, the instance is gone and must be dropped.
    pub fn cleanup(&mut self) -> crate::Result<()> {
        if self.poisoned {
            return Err(anyhow!("this instance is poisoned").into());
        }

        self.store.set_fuel(u64::MAX)?;

        Ok(())
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
