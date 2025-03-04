use std::any::Any;

use gateway_config::{HooksWasiConfig, WasiExtensionsConfig};
use wasmtime::{
    Store,
    component::{ComponentNamedList, Instance, Lift, Lower, TypedFunc},
};

use crate::{
    AccessLogSender, ComponentLoader,
    config::{build_extensions_context, build_hooks_context},
    state::WasiState,
};

pub mod hooks;

fn initialize_hooks_store(
    config: &HooksWasiConfig,
    loader: &ComponentLoader,
    access_log: AccessLogSender,
) -> crate::Result<Store<WasiState>> {
    let network_enabled = config.networking;

    let state = WasiState::new(
        build_hooks_context(config),
        access_log,
        loader.cache().clone(),
        network_enabled,
    );

    let store = Store::new(loader.engine(), state);

    Ok(store)
}

fn initialize_extensions_store(
    config: &WasiExtensionsConfig,
    loader: &ComponentLoader,
    access_log: AccessLogSender,
) -> crate::Result<Store<WasiState>> {
    let network_enabled = config.networking;

    let state = WasiState::new(
        build_extensions_context(config),
        access_log,
        loader.cache().clone(),
        network_enabled,
    );

    let store = Store::new(loader.engine(), state);

    Ok(store)
}

type FunctionCache = Vec<(&'static str, Option<Box<dyn Any + Send + Sync + 'static>>)>;

/// An instance of a component that has been loaded into the runtime.
pub struct ComponentInstance {
    store: Store<WasiState>,
    instance: Instance,
    function_cache: FunctionCache,
    poisoned: bool,
}

impl ComponentInstance {
    pub async fn new(loader: &ComponentLoader, access_log: AccessLogSender) -> crate::Result<Self> {
        let mut store = match loader.config() {
            either::Either::Left(config) => initialize_hooks_store(config, loader, access_log)?,
            either::Either::Right((_, config)) => initialize_extensions_store(config, loader, access_log)?,
        };

        let instance = loader
            .linker()
            .instantiate_async(&mut store, loader.component())
            .await?;

        Ok(Self {
            store,
            instance,
            function_cache: Default::default(),
            poisoned: false,
        })
    }

    pub fn store_mut(&mut self) -> &mut Store<WasiState> {
        &mut self.store
    }

    pub fn get_typed_func<Params, Results>(&mut self, function_name: &'static str) -> Option<TypedFunc<Params, Results>>
    where
        Params: ComponentNamedList + Lower + Send + Sync + 'static,
        Results: ComponentNamedList + Lift + Send + Sync + 'static,
    {
        if let Some((_, cached)) = self.function_cache.iter().find(|(name, _)| *name == function_name) {
            return cached.as_ref().and_then(|func| func.downcast_ref().copied());
        }

        match self.instance.get_typed_func(&mut self.store, function_name) {
            Ok(function) => {
                tracing::debug!("instantiating the {function_name} hook Wasm function");
                self.function_cache.push((function_name, Some(Box::new(function))));
                Some(function)
            }
            Err(e) => {
                // Shouldn't happen, so we keep spamming errors to be sure it's seen.
                tracing::error!("error instantiating the {function_name} hook Wasm function: {e}");
                None
            }
        }
    }

    pub fn poison(&mut self) {
        self.poisoned = true;
    }

    pub fn poisoned(&self) -> bool {
        self.poisoned
    }
}
