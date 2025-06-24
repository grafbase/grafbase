use wasmtime::component::{Resource, ResourceType, WasmList, WasmStr};

use crate::WasiState;

pub(crate) use crate::SharedContext;

pub fn add_to_linker_impl(linker: &mut wasmtime::component::Linker<WasiState>) -> wasmtime::Result<()> {
    let mut inst = linker.instance("grafbase:sdk/shared-context")?;
    inst.resource_async(
        "shared-context",
        ResourceType::host::<SharedContext>(),
        move |mut store, rep| {
            Box::new(async move {
                store.data_mut().table.delete(Resource::<SharedContext>::new_own(rep))?;
                Ok(())
            })
        },
    )?;
    inst.func_wrap_async(
        "[method]shared-context.push-event",
        move |caller: wasmtime::StoreContextMut<'_, WasiState>,
              (ctx, name, data): (Resource<SharedContext>, WasmStr, WasmList<u8>)| {
            Box::new(async move {
                let wasi_state = caller.data();
                let ctx = wasi_state.get(&ctx)?;
                // We use WasmStr & WasmList which are references into the instance's linear
                // memory. So we only copy data if we really need it.
                ctx.event_queue.push_extension_event::<wasmtime::Error>(|| {
                    Ok(event_queue::ExtensionEvent {
                        // TODO: use extension name from the WasiState
                        extension_name: String::new(),
                        event_name: name.to_str(&caller)?.into_owned(),
                        data: data.as_le_slice(&caller).to_vec(),
                    })
                })?;
                Ok(())
            })
        },
    )?;
    Ok(())
}

// For Wasmtime bindgen, does nothing.
pub trait Host {}
impl Host for WasiState {}

pub fn add_to_linker<T, U>(
    _linker: &mut wasmtime::component::Linker<T>,
    _get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
) -> wasmtime::Result<()>
where
    U: Host + Send,
    T: Send,
{
    Ok(())
}
