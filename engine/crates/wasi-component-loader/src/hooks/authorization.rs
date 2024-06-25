use anyhow::anyhow;
use grafbase_tracing::span::GRAFBASE_TARGET;
use wasmtime::{
    component::{Resource, TypedFunc},
    Store,
};

use crate::{
    context::SharedContextMap, names::AUTHORIZATION_HOOK_FUNCTION, state::WasiState, ComponentLoader, ErrorResponse,
};

/// The hook function takes two parameters: the context and the input.
/// The context is in shared memory space and the input sent by-value to the guest.
pub(crate) type Parameters = (Resource<SharedContextMap>, Vec<String>);

/// A successful result is a vector mapping the input. If a vector item is not none,
/// it will not be returned back to the client. If the function returns an error, the
/// request execution should fail.
pub(crate) type Response = (Result<Vec<Option<ErrorResponse>>, ErrorResponse>,);

/// The authorization hook is called if the requested type uses the authorization directive.
///
/// An instance of a function to be called from the Gateway level for the request.
/// The instance is meant to be separate for every request. The instance shares a memory space
/// with the guest, and cannot be shared with multiple requests.
pub struct AuthorizationHookInstance {
    store: Store<WasiState>,
    context: Resource<SharedContextMap>,
    hook: Option<TypedFunc<Parameters, Response>>,
}

impl AuthorizationHookInstance {
    pub(crate) async fn new(loader: &ComponentLoader, context: SharedContextMap) -> crate::Result<Self> {
        let mut store = super::initialize_store(loader.config(), loader.engine())?;
        let context = store.data_mut().push_resource(context)?;

        let instance = loader
            .linker()
            .instantiate_async(&mut store, loader.component())
            .await?;

        let hook = match instance.get_typed_func(&mut store, AUTHORIZATION_HOOK_FUNCTION) {
            Ok(hook) => {
                tracing::debug!(target: GRAFBASE_TARGET, "instantized the authorization hook WASM function");

                Some(hook)
            }
            Err(e) => {
                tracing::debug!(target: GRAFBASE_TARGET, "error instantizing the authorization hook WASM function: {e}");

                None
            }
        };

        Ok(Self { store, context, hook })
    }

    pub(crate) async fn call(mut self, input: Vec<String>) -> crate::Result<Vec<Option<ErrorResponse>>> {
        let result = match self.hook {
            Some(hook) => hook.call_async(&mut self.store, (self.context, input)).await?.0?,
            None => {
                return Err(crate::Error::Internal(anyhow!(
                    "authorized hook must be defined if using the @authorization directive"
                )))
            }
        };

        Ok(result)
    }
}
