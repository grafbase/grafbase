use wasmtime::component::{Resource, ResourceType, WasmList, WasmStr};

use crate::InstanceState;

pub(crate) use crate::WasmContext as SharedContext;

impl Host for InstanceState {}

pub fn add_to_linker_impl(linker: &mut wasmtime::component::Linker<InstanceState>) -> wasmtime::Result<()> {
    let mut inst = linker.instance("grafbase:sdk/shared-context")?;
    inst.resource_async(
        "shared-context",
        ResourceType::host::<SharedContext>(),
        move |mut store, rep| {
            Box::new(async move {
                store
                    .data_mut()
                    .resources
                    .delete(Resource::<SharedContext>::new_own(rep))?;
                Ok(())
            })
        },
    )?;
    inst.func_wrap_async(
        "[method]shared-context.trace-id",
        move |_: wasmtime::StoreContextMut<'_, InstanceState>, (_,): (Resource<SharedContext>,)| {
            Box::new(async move { Ok((String::new(),)) })
        },
    )?;
    inst.func_wrap_async(
        "[method]shared-context.push-event",
        move |caller: wasmtime::StoreContextMut<'_, InstanceState>,
              (ctx, name, data): (Resource<SharedContext>, WasmStr, WasmList<u8>)| {
            Box::new(async move {
                let state = caller.data();
                let ctx = state.resources.get(&ctx)?;
                // We use WasmStr & WasmList which are references into the instance's linear
                // memory. So we only copy data if we really need it.
                ctx.event_queue.push_extension_event::<wasmtime::Error>(|| {
                    Ok(event_queue::ExtensionEvent {
                        extension_name: state.extension_name().to_string(),
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
