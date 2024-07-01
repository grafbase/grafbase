use anyhow::anyhow;
use grafbase_tracing::span::GRAFBASE_TARGET;
use wasmtime::{
    component::{ComponentNamedList, Instance, Lift, Lower, Resource, TypedFunc},
    Store,
};

use crate::{
    context::SharedContextMap,
    names::{AUTHORIZATION_HOOK_FUNCTION, COMPONENT_AUTHORIZATION},
    state::WasiState,
    ComponentLoader, ErrorResponse,
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
    instance: Instance,
    poisoned: bool,
}

impl AuthorizationHookInstance {
    /// Creates a new instance of the authorization hook
    pub async fn new(loader: &ComponentLoader) -> crate::Result<Self> {
        let mut store = super::initialize_store(loader.config(), loader.engine())?;

        let instance = loader
            .linker()
            .instantiate_async(&mut store, loader.component())
            .await?;

        Ok(Self {
            store,
            instance,
            poisoned: false,
        })
    }

    /// Calls the authorization hook
    pub async fn call(
        &mut self,
        context: SharedContextMap,
        input: Vec<String>,
    ) -> crate::Result<Vec<Option<ErrorResponse>>> {
        match self.get_hook::<Parameters, Response>(AUTHORIZATION_HOOK_FUNCTION) {
            Some(hook) => {
                let context = self.store.data_mut().push_resource(context)?;
                let context_rep = context.rep();

                let result = hook.call_async(&mut self.store, (context, input)).await;

                // We check if the hook call trapped, and if so we mark the instance poisoned.
                //
                // If no traps, we mark this hook so it can be called again.
                if result.is_err() {
                    self.poisoned = true;
                } else {
                    hook.post_return_async(&mut self.store).await?;
                }

                let result = result?.0;

                // This is a bit ugly because we don't need it, but we need to clean the shared
                // resources before exiting or this will leak RAM.
                let _: SharedContextMap = self.store.data_mut().take_resource(context_rep)?;

                let result = result?;

                Ok(result)
            }
            None => Err(crate::Error::Internal(anyhow!(
                "authorized hook must be defined if using the @authorization directive"
            ))),
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

    /// A generic get hook we can use to find a different function from the interface.
    fn get_hook<I, O>(&mut self, function_name: &str) -> Option<TypedFunc<I, O>>
    where
        I: ComponentNamedList + Lower,
        O: ComponentNamedList + Lift,
    {
        let mut exports = self.instance.exports(&mut self.store);
        let mut root = exports.root();

        let Some(mut interface) = root.instance(COMPONENT_AUTHORIZATION) else {
            tracing::debug!(target: GRAFBASE_TARGET, "could not find export for authorization interface");
            return None;
        };

        match interface.typed_func(function_name) {
            Ok(hook) => {
                tracing::debug!(target: GRAFBASE_TARGET, "instantized the authorization hook WASM function");
                Some(hook)
            }
            Err(e) => {
                tracing::debug!(target: GRAFBASE_TARGET, "error instantizing the authorization hook WASM function: {e}");
                None
            }
        }
    }
}
