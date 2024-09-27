use std::any::Any;
use std::future::Future;
use std::sync::RwLock;

use anyhow::anyhow;
use tracing::Instrument;
use wasmtime::{
    component::{Component, ComponentNamedList, Instance, Lift, Lower, Resource, TypedFunc},
    Engine, Store,
};

use crate::{config::build_wasi_context, state::WasiState, ComponentLoader, Config, SharedContext};

pub(crate) mod authorization;
pub(crate) mod gateway;
pub(crate) mod response;
pub(crate) mod subgraph;

/// A trait for components that can be recycled
pub trait RecycleableComponentInstance: Sized + Send + 'static {
    /// Creates a new instance of the component.
    fn new(loader: &ComponentLoader) -> impl Future<Output = crate::Result<Self>> + Send;

    /// Resets the store to the original state. This must be called if wanting to reuse this instance.
    fn recycle(&mut self) -> crate::Result<()>;
}

/// A macro to define a component instance.
///
/// This macro generates a struct representing a component instance along with the necessary
/// implementations for `Debug`, `Deref`, and `DerefMut` traits. It also implements the
/// `RecycleableComponentInstance` trait for the generated struct, allowing for creation and
/// recycling of the component instance.
///
/// # Arguments
///
/// * `$ty` - The identifier for the generated struct.
/// * `$name` - The name of the component that will be instantiated.
macro_rules! component_instance {
    ($ty:ident: $name:expr) => {
        /// A struct representing an instance of the component.
        ///
        /// This struct wraps the `ComponentInstance` and provides the necessary
        /// implementations for various traits, including `Debug`, `Deref`, and
        /// `DerefMut`. It also implements the `RecycleableComponentInstance` trait,
        /// allowing for the creation and recycling of the component instance.
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

/// Initializes a new `Store<WasiState>` with the given configuration and engine.
///
/// # Arguments
///
/// * `config` - A reference to the configuration used to build the WASI context.
/// * `engine` - A reference to the Wasmtime engine used for creating the store.
///
/// # Returns
///
/// A `Result` containing a `Store<WasiState>` on success, or an error if initialization fails.
///
/// This function creates a new `WasiState` using the provided configuration, initializes the store
/// with the maximum fuel, and sets a yield interval how often to allow the main thread to be yielded.
fn initialize_store(config: &Config, engine: &Engine) -> crate::Result<Store<WasiState>> {
    let state = WasiState::new(build_wasi_context(config));

    let mut store = Store::new(engine, state);
    store.set_fuel(u64::MAX)?;

    // make this smaller to yield to the main thread more often
    store.fuel_async_yield_interval(Some(10000))?;

    Ok(store)
}

type FunctionCache = RwLock<Vec<(&'static str, Option<Box<dyn Any + Send + Sync + 'static>>)>>;

pub struct ComponentInstance {
    /// The store associated with the WASI state.
    store: Store<WasiState>,
    /// The instance of the component.
    instance: Instance,
    /// The guest component.
    component: Component,
    /// The name of the interface this component implements.
    interface_name: &'static str,
    /// A cache for storing instantiated hook functions.
    function_cache: FunctionCache,
    /// Indicates whether the instance has encountered a fatal error.
    poisoned: bool,
}

impl ComponentInstance {
    /// Creates a new instance of the component.
    ///
    /// # Arguments
    ///
    /// * `loader` - A reference to the `ComponentLoader` used to load the component.
    /// * `interface_name` - The name of the interface this component implements.
    ///
    /// # Returns
    ///
    /// A `Result` containing the newly created component instance on success, or an error on failure.
    async fn new(loader: &ComponentLoader, interface_name: &'static str) -> crate::Result<Self> {
        let mut store = initialize_store(loader.config(), loader.engine())?;

        let instance = loader
            .linker()
            .instantiate_async(&mut store, loader.component())
            .await?;

        let component = loader.component().clone();

        Ok(Self {
            store,
            instance,
            component,
            interface_name,
            function_cache: Default::default(),
            poisoned: false,
        })
    }

    /// Calls a function with one input argument and no output.
    ///
    /// # Type Parameters
    ///
    /// * `A1` - The type of the first argument.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function to call.
    /// * `context` - A shared context resource.
    /// * `arg` - The first argument to pass to the function.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure. If the function call is successful, it returns `Ok(())`.
    async fn call1_without_output<A1>(
        &mut self,
        name: &'static str,
        context: SharedContext,
        arg: A1,
    ) -> crate::Result<()>
    where
        (Resource<SharedContext>, A1): ComponentNamedList + Lower + Send + Sync + 'static,
    {
        let Some(hook) = self.get_hook::<(Resource<SharedContext>, A1), ()>(name) else {
            return Ok(());
        };

        let span = tracing::info_span!("hook", "otel.name" = name);
        let context = self.store.data_mut().push_resource(context)?;
        let context_rep = context.rep();

        let result = hook.call_async(&mut self.store, (context, arg)).instrument(span).await;

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

    /// Calls a function with one input argument and one output.
    ///
    /// # Type Parameters
    ///
    /// * `A1` - The type of the first argument.
    /// * `R` - The type of the output.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function to call.
    /// * `context` - A shared context resource.
    /// * `arg` - The first argument to pass to the function.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<R>`. If the function call is successful, it returns `Ok(Some(result))`,
    /// where `result` is the output of the function. If the function call fails, it returns an error. If the
    /// function does not exist, it returns `Ok(None)`.
    async fn call1_one_output<A1, R>(
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

        let span = tracing::info_span!("hook", "otel.name" = name);
        let context = self.store.data_mut().push_resource(context)?;
        let context_rep = context.rep();

        let result = hook.call_async(&mut self.store, (context, arg)).instrument(span).await;

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

    /// Calls a function with two input arguments and one output.
    ///
    /// # Type Parameters
    ///
    /// * `A1` - The type of the first argument.
    /// * `A2` - The type of the second argument.
    /// * `R` - The type of the output.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function to call.
    /// * `context` - A shared context resource.
    /// * `args` - A tuple containing the two arguments to pass to the function.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<R>`. If the function call is successful, it returns `Ok(Some(result))`,
    /// where `result` is the output of the function. If the function call fails, it returns an error. If the
    /// function does not exist, it returns `Ok(None)`.
    async fn call2_one_output<A1, A2, R>(
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

        let span = tracing::info_span!("hook", "otel.name" = name);
        let context = self.store.data_mut().push_resource(context)?;
        let context_rep = context.rep();

        let result = hook
            .call_async(&mut self.store, (context, args.0, args.1))
            .instrument(span)
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

    /// Calls a function with three input arguments and one output.
    ///
    /// # Type Parameters
    ///
    /// * `A1` - The type of the first argument.
    /// * `A2` - The type of the second argument.
    /// * `A3` - The type of the third argument.
    /// * `R` - The type of the output.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function to call.
    /// * `context` - A shared context resource.
    /// * `args` - A tuple containing the three arguments to pass to the function.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option<R>`. If the function call is successful, it returns `Ok(Some(result))`,
    /// where `result` is the output of the function. If the function call fails, it returns an error. If the
    /// function does not exist, it returns `Ok(None)`.
    async fn call3_one_output<A1, A2, A3, R>(
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

        let span = tracing::info_span!("hook", "otel.name" = name);
        let context = self.store.data_mut().push_resource(context)?;
        let context_rep = context.rep();

        let result = hook
            .call_async(&mut self.store, (context, args.0, args.1, args.2))
            .instrument(span)
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

    /// Retrieves a typed function (hook) by its name from the component instance.
    ///
    /// # Type Parameters
    ///
    /// * `I` - The input type for the function.
    /// * `O` - The output type for the function.
    ///
    /// # Arguments
    ///
    /// * `function_name` - The name of the function to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<TypedFunc<I, O>>`, which is `Some` if the function was found and can be cast to the expected types,
    /// or `None` if the function does not exist or could not be retrieved.
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

        let Some((_, interface_idx)) = self.component.export_index(None, self.interface_name) else {
            tracing::debug!("could not find export for {} interface", self.interface_name);
            self.function_cache.write().unwrap().push((function_name, None));

            return None;
        };

        let Some((_, func_idx)) = self.component.export_index(Some(&interface_idx), function_name) else {
            tracing::debug!(
                "could not find function {} in interface {}",
                function_name,
                self.interface_name
            );

            self.function_cache.write().unwrap().push((function_name, None));
            return None;
        };

        match self.instance.get_typed_func(&mut self.store, func_idx) {
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

    /// Resets the component instance for reuse.
    ///
    /// This function sets the fuel of the store to its maximum value, allowing
    /// the instance to be recycled for future calls. If the instance has
    /// encountered a fatal error (marked as poisoned), this function will
    /// return an error instead.
    ///
    /// This function must be called before reusing for another request.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure. On success, it returns `Ok(())`.
    /// On failure, it returns an error if the instance is poisoned.
    pub fn recycle(&mut self) -> crate::Result<()> {
        if self.poisoned {
            return Err(anyhow!("this instance is poisoned").into());
        }

        self.store.set_fuel(u64::MAX)?;

        Ok(())
    }
}
