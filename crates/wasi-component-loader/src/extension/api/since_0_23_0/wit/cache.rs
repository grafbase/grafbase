use std::{sync::Arc, time::Duration};

use wasmtime::component::{Resource, ResourceType, WasmStr};

use crate::InstanceState;

pub use crate::resources::Cache;

impl Host for InstanceState {}

pub fn add_to_linker_impl(linker: &mut wasmtime::component::Linker<InstanceState>) -> wasmtime::Result<()> {
    let mut inst = linker.instance("grafbase:sdk/cache")?;
    inst.resource_async("cache", ResourceType::host::<Cache>(), move |mut store, rep| {
        Box::new(async move {
            store.data_mut().resources.delete(Resource::<Cache>::new_own(rep))?;
            Ok(())
        })
    })?;
    inst.func_wrap_async(
        "[static]cache.init",
        move |mut caller: wasmtime::StoreContextMut<'_, InstanceState>,
              (name, size, ttl_ms): (String, u32, Option<u64>)| {
            Box::new(async move {
                let state = caller.data_mut();
                let cache = state
                    .caches
                    .entry(name)
                    .or_insert_with(|| Cache::new(size as usize, ttl_ms.map(Duration::from_millis)))
                    .clone();
                let cache = state.resources.push(cache)?;
                Ok((cache,))
            })
        },
    )?;
    inst.func_wrap_async(
        "[method]cache.get-or-reserve",
        move |caller: wasmtime::StoreContextMut<'_, InstanceState>,
              (cache, key, timeout_ms): (wasmtime::component::Resource<Cache>, WasmStr, u64)| {
            Box::new(async move {
                let state = caller.data();
                let cache = state.resources.get(&cache)?;

                let key = key.to_str(&caller)?;
                let value = cache.get(key.as_ref(), Duration::from_millis(timeout_ms)).await;

                Ok((value,))
            })
        },
    )?;
    inst.func_wrap_async(
        "[method]cache.insert",
        move |caller: wasmtime::StoreContextMut<'_, InstanceState>,
              (cache, key, value): (wasmtime::component::Resource<Cache>, WasmStr, Arc<[u8]>)| {
            Box::new(async move {
                let state = caller.data();
                let cache = state.resources.get(&cache)?;

                let key = key.to_str(&caller)?;
                cache.insert(key.as_ref(), value).await;

                Ok(())
            })
        },
    )?;
    inst.func_wrap_async(
        "[method]cache.remove",
        move |caller: wasmtime::StoreContextMut<'_, InstanceState>,
              (cache, key): (wasmtime::component::Resource<Cache>, WasmStr)| {
            Box::new(async move {
                let state = caller.data();
                let cache = state.resources.get(&cache)?;

                let key = key.to_str(&caller)?;
                cache.remove(key.as_ref());

                Ok(())
            })
        },
    )?;
    Ok(())
}

// Typical Wasmtime bindgen! macro generated stuff
// It's really just unnecessary work to implement this when we can just call the function with the
// real type.
pub trait Host: Send + ::core::marker::Send {}
impl<_T: Host + ?Sized + Send> Host for &mut _T {}
pub fn add_to_linker<T, D>(
    _linker: &mut wasmtime::component::Linker<T>,
    _host_getter: fn(&mut T) -> D::Data<'_>,
) -> wasmtime::Result<()>
where
    D: wasmtime::component::HasData,
    for<'a> D::Data<'a>: Host,
    T: 'static + Send,
{
    Ok(())
}
