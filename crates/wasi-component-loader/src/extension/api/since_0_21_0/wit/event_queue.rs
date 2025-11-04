use wasmtime::component::{Resource, ResourceType, WasmList, WasmStr};

use crate::{InstanceState, extension::api::since_0_19_0::wit::event_types};

pub use crate::resources::EventQueueResource as EventQueue;

impl Host for InstanceState {}

pub fn add_to_linker_impl(linker: &mut wasmtime::component::Linker<InstanceState>) -> wasmtime::Result<()> {
    let mut inst = linker.instance("grafbase:sdk/event-queue")?;
    inst.resource_async(
        "event-queue",
        ResourceType::host::<EventQueue>(),
        move |mut store, rep| {
            Box::new(async move {
                store
                    .data_mut()
                    .resources
                    .delete(Resource::<EventQueue>::new_own(rep))?;
                Ok(())
            })
        },
    )?;
    inst.func_wrap_async(
        "[method]event-queue.push",
        move |caller: wasmtime::StoreContextMut<'_, InstanceState>,
              (event_queue, name, data): (Resource<EventQueue>, WasmStr, WasmList<u8>)| {
            Box::new(async move {
                let state = caller.data();
                let event_queue = state.resources.get(&event_queue)?;
                // We use WasmStr & WasmList which are references into the instance's linear
                // memory. So we only copy data if we really need it.
                event_queue.push_extension_event::<wasmtime::Error>(|| {
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
    inst.func_wrap_async(
        "[method]event-queue.pop",
        move |mut caller: wasmtime::StoreContextMut<'_, InstanceState>, (event_queue,): (Resource<EventQueue>,)| {
            Box::new(async move {
                let state = caller.data_mut();
                let event_queue = state.resources.get(&event_queue)?;
                match event_queue.pop() {
                    Some(event) => Ok((Some(event_types::convert_event(state, event)?),)),
                    None => Ok((None,)),
                }
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
