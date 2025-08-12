use std::sync::Arc;

use event_queue::EventQueue;
use runtime::extension::ExtensionRequestContext;
use wasmtime::component::{Resource, ResourceType, WasmList, WasmStr};

use crate::InstanceState;
use crate::extension::api::wit;

pub(crate) struct HostContext(Arc<EventQueue>);

pub(crate) use engine::{
    EngineOperationContext as AuthorizedOperationContext, EngineRequestContext as AuthenticatedRequestContext,
};
pub(crate) struct RequestContext {
    pub hooks_context: Arc<[u8]>,
}

impl From<&ExtensionRequestContext> for RequestContext {
    fn from(ctx: &ExtensionRequestContext) -> Self {
        Self {
            hooks_context: ctx.hooks_context.clone(),
        }
    }
}

impl From<&ExtensionRequestContext> for HostContext {
    fn from(ctx: &ExtensionRequestContext) -> Self {
        Self(ctx.event_queue.clone())
    }
}

impl From<&engine::EngineRequestContext> for HostContext {
    fn from(ctx: &engine::EngineRequestContext) -> Self {
        Self(ctx.extension().event_queue.clone())
    }
}

impl From<&engine::EngineOperationContext> for HostContext {
    fn from(ctx: &engine::EngineOperationContext) -> Self {
        Self(ctx.extension().event_queue.clone())
    }
}

impl From<Arc<EventQueue>> for HostContext {
    fn from(event_queue: Arc<EventQueue>) -> Self {
        Self(event_queue)
    }
}

impl From<&Arc<EventQueue>> for HostContext {
    fn from(event_queue: &Arc<EventQueue>) -> Self {
        Self(event_queue.clone())
    }
}

impl Host for InstanceState {}

pub fn add_to_linker_impl(linker: &mut wasmtime::component::Linker<InstanceState>) -> wasmtime::Result<()> {
    let mut inst = linker.instance("grafbase:sdk/context")?;

    // === HostContext ===
    inst.resource_async(
        "host-context",
        ResourceType::host::<HostContext>(),
        move |mut store, rep| {
            Box::new(async move {
                store
                    .data_mut()
                    .resources
                    .delete(Resource::<HostContext>::new_own(rep))?;
                Ok(())
            })
        },
    )?;
    inst.func_wrap_async(
        "[method]host-context.push-event",
        move |caller: wasmtime::StoreContextMut<'_, InstanceState>,
              (ctx, name, data): (Resource<HostContext>, WasmStr, WasmList<u8>)| {
            Box::new(async move {
                let state = caller.data();
                let ctx = state.resources.get(&ctx)?;
                // We use WasmStr & WasmList which are references into the instance's linear
                // memory. So we only copy data if we really need it.
                ctx.0.push_extension_event::<wasmtime::Error>(|| {
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

    // === RequestContext ===
    inst.resource_async(
        "request-context",
        ResourceType::host::<RequestContext>(),
        move |mut store, rep| {
            Box::new(async move {
                store
                    .data_mut()
                    .resources
                    .delete(Resource::<RequestContext>::new_own(rep))?;
                Ok(())
            })
        },
    )?;
    inst.func_wrap_async(
        "[method]request-context.hooks-context",
        move |caller: wasmtime::StoreContextMut<'_, InstanceState>, (ctx,): (Resource<RequestContext>,)| {
            Box::new(async move {
                let state = caller.data();
                let ctx = state.resources.get(&ctx)?;
                Ok((ctx.hooks_context.clone(),))
            })
        },
    )?;

    // === AuthenticatedRequestContext ===
    inst.resource_async(
        "authenticated-request-context",
        ResourceType::host::<AuthenticatedRequestContext>(),
        move |mut store, rep| {
            Box::new(async move {
                store
                    .data_mut()
                    .resources
                    .delete(Resource::<AuthenticatedRequestContext>::new_own(rep))?;
                Ok(())
            })
        },
    )?;
    inst.func_wrap_async(
        "[method]authenticated-request-context.hooks-context",
        move |caller: wasmtime::StoreContextMut<'_, InstanceState>,
              (ctx,): (Resource<AuthenticatedRequestContext>,)| {
            Box::new(async move {
                let state = caller.data();
                let ctx = state.resources.get(&ctx)?;
                Ok((ctx.extension().hooks_context.clone(),))
            })
        },
    )?;
    inst.func_wrap_async(
        "[method]authenticated-request-context.token",
        move |caller: wasmtime::StoreContextMut<'_, InstanceState>,
              (ctx,): (Resource<AuthenticatedRequestContext>,)| {
            Box::new(async move {
                let state = caller.data();
                let ctx = state.resources.get(&ctx)?;
                Ok((wit::Token::from(ctx.token().clone()),))
            })
        },
    )?;

    // === AuthorizedOperationContext ===
    inst.resource_async(
        "authorized-operation-context",
        ResourceType::host::<AuthorizedOperationContext>(),
        move |mut store, rep| {
            Box::new(async move {
                store
                    .data_mut()
                    .resources
                    .delete(Resource::<AuthorizedOperationContext>::new_own(rep))?;
                Ok(())
            })
        },
    )?;
    inst.func_wrap_async(
        "[method]authorized-operation-context.hooks-context",
        move |caller: wasmtime::StoreContextMut<'_, InstanceState>, (ctx,): (Resource<AuthorizedOperationContext>,)| {
            Box::new(async move {
                let state = caller.data();
                let ctx = state.resources.get(&ctx)?;
                Ok((ctx.extension().hooks_context.clone(),))
            })
        },
    )?;
    inst.func_wrap_async(
        "[method]authorized-operation-context.token",
        move |caller: wasmtime::StoreContextMut<'_, InstanceState>, (ctx,): (Resource<AuthorizedOperationContext>,)| {
            Box::new(async move {
                let state = caller.data();
                let ctx = state.resources.get(&ctx)?;
                Ok((wit::Token::from(ctx.token().clone()),))
            })
        },
    )?;
    inst.func_wrap_async(
        "[method]authorized-operation-context.authorization-context",
        move |caller: wasmtime::StoreContextMut<'_, InstanceState>,
              (ctx, key): (Resource<AuthorizedOperationContext>, Option<String>)| {
            Box::new(async move {
                let state = caller.data();
                let ctx = state.resources.get(&ctx)?;
                let catalog = &state.catalog;
                Ok((match key {
                    Some(key) => Ok(ctx
                        .authorization_context()
                        .iter()
                        .find_map(|(id, bytes)| {
                            if catalog[*id].manifest.name() == key {
                                Some(bytes.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default()),
                    None => {
                        if ctx.authorization_context().len() <= 1 {
                            Ok(ctx
                                .authorization_context()
                                .first()
                                .map(|(_, bytes)| bytes.clone())
                                .unwrap_or_default())
                        } else {
                            Err("Multiple authorization contexts provided, but no key specified".to_string())
                        }
                    }
                },))
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
