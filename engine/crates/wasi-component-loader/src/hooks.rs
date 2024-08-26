use std::any::Any;
use std::future::Future;
use std::sync::RwLock;

use anyhow::anyhow;
use wasmtime::{
    component::{ComponentNamedList, Instance, Lift, Lower, Resource, TypedFunc},
    Engine, Store,
};

use crate::{config::build_wasi_context, state::WasiState, ComponentLoader, Config, SharedContext};

pub(crate) mod authorization;
pub(crate) mod gateway;
pub(crate) mod response;
pub(crate) mod subgraph;

/// A trait for components that can be recycled
pub trait RecycleableComponentInstance: Sized + Send + 'static {
    /// Creates a new instance of the component
    fn new(loader: &ComponentLoader) -> impl Future<Output = crate::Result<Self>> + Send;
    /// Resets the store to the original state. This must be called if wanting to reuse this instance.
    fn recycle(&mut self) -> crate::Result<()>;
}

macro_rules! component_instance {
    ($ty:ident: $name:expr) => {
        /// A component instance
        pub struct $ty(ComponentInstance);

        impl std::fmt::Debug for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", stringify!($ty))
            }
        }

        impl std::ops::Deref for $ty {
            type Target = ComponentInstance;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $ty {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl $crate::RecycleableComponentInstance for $ty {
            async fn new(loader: &ComponentLoader) -> $crate::Result<Self> {
                ComponentInstance::new(loader, $name).await.map(Self)
            }

            fn recycle(&mut self) -> $crate::Result<()> {
                self.0.recycle()
            }
        }
    };
}

pub(crate) use component_instance;

/// Generic initialization of WASI components for all hooks.
fn initialize_store(config: &Config, engine: &Engine) -> crate::Result<Store<WasiState>> {
    let state = WasiState::new(build_wasi_context(config));

    let mut store = Store::new(engine, state);
    store.set_fuel(u64::MAX)?;

    // make this smaller to yield to the main thread more often
    store.fuel_async_yield_interval(Some(10000))?;

    Ok(store)
}

type FunctionCache = RwLock<Vec<(&'static str, Option<Box<dyn Any + Send + Sync + 'static>>)>>;

/// Component instance for hooks
pub struct ComponentInstance {
    store: Store<WasiState>,
    instance: Instance,
    interface_name: &'static str,
    function_cache: FunctionCache,
    poisoned: bool,
}

impl ComponentInstance {
    /// Creates a new instance of the authorization hook
    async fn new(loader: &ComponentLoader, interface_name: &'static str) -> crate::Result<Self> {
        let mut store = initialize_store(loader.config(), loader.engine())?;

        let instance = loader
            .linker()
            .instantiate_async(&mut store, loader.component())
            .await?;

        Ok(Self {
            store,
            instance,
            interface_name,
            function_cache: Default::default(),
            poisoned: false,
        })
    }

    async fn call1_effect0<A1>(&mut self, name: &'static str, context: SharedContext, arg: A1) -> crate::Result<()>
    where
        (Resource<SharedContext>, A1): ComponentNamedList + Lower + Send + Sync + 'static,
    {
        let Some(hook) = self.get_hook::<(Resource<SharedContext>, A1), ()>(name) else {
            return Ok(());
        };

        let context = self.store.data_mut().push_resource(context)?;
        let context_rep = context.rep();

        let result = hook.call_async(&mut self.store, (context, arg)).await;

        // We check if the hook call trapped, and if so we mark the instance poisoned.
        //
        // If no traps, we mark this hook so it can be called again.
        if result.is_err() {
            self.poisoned = true;
        } else {
            hook.post_return_async(&mut self.store).await?;
        }

        result?;

        // This is a bit ugly because we don't need it, but we need to clean the shared
        // resources before exiting or this will leak RAM.
        let _: SharedContext = self.store.data_mut().take_resource(context_rep)?;

        Ok(())
    }

    async fn call1_effect1<A1, R>(
        &mut self,
        name: &'static str,
        context: SharedContext,
        arg: A1,
    ) -> crate::Result<Option<R>>
    where
        (Resource<SharedContext>, A1): ComponentNamedList + Lower + Send + Sync + 'static,
        (R,): ComponentNamedList + Lift + Send + Sync + 'static,
    {
        let Some(hook) = self.get_hook::<(Resource<SharedContext>, A1), (R,)>(name) else {
            return Ok(None);
        };

        let context = self.store.data_mut().push_resource(context)?;
        let context_rep = context.rep();

        let result = hook.call_async(&mut self.store, (context, arg)).await;

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
        let _: SharedContext = self.store.data_mut().take_resource(context_rep)?;

        Ok(Some(result))
    }

    async fn call2_effect1<A1, A2, R>(
        &mut self,
        name: &'static str,
        context: SharedContext,
        args: (A1, A2),
    ) -> crate::Result<Option<R>>
    where
        (Resource<SharedContext>, A1, A2): ComponentNamedList + Lower + Send + Sync + 'static,
        (R,): ComponentNamedList + Lift + Send + Sync + 'static,
    {
        let Some(hook) = self.get_hook::<(Resource<SharedContext>, A1, A2), (R,)>(name) else {
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
        let _: SharedContext = self.store.data_mut().take_resource(context_rep)?;

        Ok(Some(result))
    }

    async fn call3_effect1<A1, A2, A3, R>(
        &mut self,
        name: &'static str,
        context: SharedContext,
        args: (A1, A2, A3),
    ) -> crate::Result<Option<R>>
    where
        (Resource<SharedContext>, A1, A2, A3): ComponentNamedList + Lower + Send + Sync + 'static,
        (R,): ComponentNamedList + Lift + Send + Sync + 'static,
    {
        let Some(hook) = self.get_hook::<(Resource<SharedContext>, A1, A2, A3), (R,)>(name) else {
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
        let _: SharedContext = self.store.data_mut().take_resource(context_rep)?;

        Ok(Some(result))
    }

    /// A generic get hook we can use to find a different function from the interface.
    fn get_hook<I, O>(&mut self, function_name: &'static str) -> Option<TypedFunc<I, O>>
    where
        I: ComponentNamedList + Lower + Send + Sync + 'static,
        O: ComponentNamedList + Lift + Send + Sync + 'static,
    {
        if let Some((_, cached)) = self
            .function_cache
            .read()
            .unwrap()
            .iter()
            .find(|(name, _)| *name == function_name)
        {
            return cached.as_ref().and_then(|func| func.downcast_ref().copied());
        }

        let mut exports = self.instance.exports(&mut self.store);
        let mut root = exports.root();

        let Some(mut interface) = root.instance(self.interface_name) else {
            tracing::debug!("could not find export for {} interface", self.interface_name);
            self.function_cache.write().unwrap().push((function_name, None));
            return None;
        };

        match interface.typed_func(function_name) {
            Ok(hook) => {
                tracing::debug!("instantized the {function_name} hook Wasm function");
                self.function_cache
                    .write()
                    .unwrap()
                    .push((function_name, Some(Box::new(hook))));
                Some(hook)
            }
            Err(e) => {
                // Shouldn't happen, so we keep spamming errors to be sure it's seen.
                tracing::error!("error instantizing the {function_name} hook Wasm function: {e}");
                None
            }
        }
    }

    /// Resets the store to the original state. This must be called if wanting to reuse this instance.
    ///
    /// If the cleanup fails, the instance is gone and must be dropped.
    pub fn recycle(&mut self) -> crate::Result<()> {
        if self.poisoned {
            return Err(anyhow!("this instance is poisoned").into());
        }

        self.store.set_fuel(u64::MAX)?;

        Ok(())
    }
}
