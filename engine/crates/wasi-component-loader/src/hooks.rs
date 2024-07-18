use std::sync::RwLock;

use anyhow::anyhow;
use grafbase_tracing::span::GRAFBASE_TARGET;
use wasmtime::{
    component::{ComponentNamedList, Instance, Lift, Lower, Resource, TypedFunc},
    Engine, Store,
};

use crate::{state::WasiState, ComponentLoader, Config, SharedContextMap};

pub(crate) mod authorization;
pub(crate) mod gateway;
pub(crate) mod subgraph;

/// Generic initialization of WASI components for all hooks.
fn initialize_store(config: &Config, engine: &Engine) -> crate::Result<Store<WasiState>> {
    let state = WasiState::new(config.wasi_context());

    let mut store = Store::new(engine, state);
    store.set_fuel(u64::MAX)?;

    // make this smaller to yield to the main thread more often
    store.fuel_async_yield_interval(Some(10000))?;

    Ok(store)
}

type FunctionCache = RwLock<Vec<(&'static str, Box<dyn std::any::Any + Send + Sync>)>>;

pub struct ComponentInstance {
    store: Store<WasiState>,
    instance: Instance,
    export_name: &'static str,
    function_cache: FunctionCache,
    poisoned: bool,
}

impl ComponentInstance {
    /// Creates a new instance of the authorization hook
    async fn new(loader: &ComponentLoader, export_name: &'static str) -> crate::Result<Self> {
        let mut store = initialize_store(loader.config(), loader.engine())?;

        let instance = loader
            .linker()
            .instantiate_async(&mut store, loader.component())
            .await?;

        Ok(Self {
            store,
            instance,
            export_name,
            function_cache: Default::default(),
            poisoned: false,
        })
    }

    async fn call2<A1, A2, R>(
        &mut self,
        name: &'static str,
        context: SharedContextMap,
        args: (A1, A2),
    ) -> crate::Result<Option<R>>
    where
        (Resource<SharedContextMap>, A1, A2): ComponentNamedList + Lower + Send + Sync + 'static,
        (R,): ComponentNamedList + Lift + Send + Sync + 'static,
    {
        let Some(hook) = self.get_hook::<(Resource<SharedContextMap>, A1, A2), (R,)>(name) else {
            return Ok(None);
        };

        let context = self.store.data_mut().push_resource(context)?;
        let context_rep = context.rep();

        let result = hook.call_async(&mut self.store, (context, args.0, args.1)).await;

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

        Ok(Some(result))
    }

    async fn call3<A1, A2, A3, R>(
        &mut self,
        name: &'static str,
        context: SharedContextMap,
        args: (A1, A2, A3),
    ) -> crate::Result<Option<R>>
    where
        (Resource<SharedContextMap>, A1, A2, A3): ComponentNamedList + Lower + Send + Sync + 'static,
        (R,): ComponentNamedList + Lift + Send + Sync + 'static,
    {
        let Some(hook) = self.get_hook::<(Resource<SharedContextMap>, A1, A2, A3), (R,)>(name) else {
            return Ok(None);
        };

        let context = self.store.data_mut().push_resource(context)?;
        let context_rep = context.rep();

        let result = hook
            .call_async(&mut self.store, (context, args.0, args.1, args.2))
            .await;

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

        Ok(Some(result))
    }

    /// A generic get hook we can use to find a different function from the interface.
    fn get_hook<I, O>(&mut self, function_name: &'static str) -> Option<TypedFunc<I, O>>
    where
        I: ComponentNamedList + Lower + Send + Sync + 'static,
        O: ComponentNamedList + Lift + Send + Sync + 'static,
    {
        if let Some(func) = self
            .function_cache
            .read()
            .unwrap()
            .iter()
            .filter(|(name, _)| *name == function_name)
            .find_map(|(_, cached)| cached.downcast_ref::<TypedFunc<I, O>>())
        {
            return Some(*func);
        }
        let mut exports = self.instance.exports(&mut self.store);
        let mut root = exports.root();

        let Some(mut interface) = root.instance(self.export_name) else {
            tracing::debug!(target: GRAFBASE_TARGET, "could not find export for authorization interface");
            return None;
        };

        match interface.typed_func(function_name) {
            Ok(hook) => {
                tracing::debug!(target: GRAFBASE_TARGET, "instantized the authorization hook WASM function");
                self.function_cache
                    .write()
                    .unwrap()
                    .push((function_name, Box::new(hook)));
                Some(hook)
            }
            Err(e) => {
                tracing::debug!(target: GRAFBASE_TARGET, "error instantizing the authorization hook WASM function: {e}");
                None
            }
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
