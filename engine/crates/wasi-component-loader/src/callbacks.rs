use wasmtime::{Engine, Store};

use crate::{state::WasiState, Config};

pub(crate) mod gateway;

/// Generic initialization of WASI components for all callbacks.
fn initialize_store(config: &Config, engine: &Engine) -> crate::Result<Store<WasiState>> {
    let state = WasiState::new(config.wasi_context());

    let mut store = Store::new(engine, state);
    store.set_fuel(u64::MAX)?;

    // make this smaller to yield to the main thread more often
    store.fuel_async_yield_interval(Some(10000))?;

    Ok(store)
}
