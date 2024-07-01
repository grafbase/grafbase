use anyhow::anyhow;
use grafbase_tracing::span::GRAFBASE_TARGET;
use wasmtime::{
    component::{ComponentNamedList, ComponentType, Instance, Lift, Lower, Resource, TypedFunc},
    Store,
};

use crate::{
    context::SharedContextMap,
    names::{
        AUTHORIZE_EDGE_PRE_EXECUTION_HOOK_FUNCTION, AUTHORIZE_NODE_PRE_EXECUTION_HOOK_FUNCTION, COMPONENT_AUTHORIZATION,
    },
    state::WasiState,
    ComponentLoader, ErrorResponse,
};

/// Defines an edge in an authorization hook.
#[derive(Lower, ComponentType)]
#[component(record)]
pub struct EdgeDefinition {
    /// The name of the type this edge is part of
    #[component(name = "parent-type-name")]
    pub parent_type_name: String,
    /// The name of the field of this edge
    #[component(name = "field-name")]
    pub field_name: String,
}

/// Defines a node in an authorization hook.
#[derive(Lower, ComponentType)]
#[component(record)]
pub struct NodeDefinition {
    /// The name of the type of this node
    #[component(name = "type-name")]
    pub type_name: String,
}

pub(crate) type EdgePreParameters = (Resource<SharedContextMap>, EdgeDefinition, String, String);
pub(crate) type EdgePreResponse = (Result<(), ErrorResponse>,);

pub(crate) type NodePreParameters = (Resource<SharedContextMap>, NodeDefinition, String);
pub(crate) type NodePreResponse = (Result<(), ErrorResponse>,);

/// The authorization hook is called if the requested type uses the authorization directive.
///
/// An instance of a function to be called from the Gateway level for the request.
/// The instance is meant to be separate for every request. The instance shares a memory space
/// with the guest, and cannot be shared with multiple requests.
pub struct AuthorizationHookInstance {
    store: Store<WasiState>,
    instance: Instance,
    poisoned: bool,
}

impl AuthorizationHookInstance {
    /// Creates a new instance of the authorization hook
    pub async fn new(loader: &ComponentLoader) -> crate::Result<Self> {
        let mut store = super::initialize_store(loader.config(), loader.engine())?;

        let instance = loader
            .linker()
            .instantiate_async(&mut store, loader.component())
            .await?;

        Ok(Self {
            store,
            instance,
            poisoned: false,
        })
    }

    /// Calls the pre authorize hook for an edge
    pub async fn authorize_edge_pre_execution(
        &mut self,
        context: SharedContextMap,
        definition: EdgeDefinition,
        arguments: String,
        metadata: String,
    ) -> crate::Result<()> {
        let Some(hook) =
            self.get_hook::<EdgePreParameters, EdgePreResponse>(AUTHORIZE_EDGE_PRE_EXECUTION_HOOK_FUNCTION)
        else {
            return Err(crate::Error::Internal(anyhow!(
                "authorize-edge-pre-execution hook must be defined if using the @authorization directive"
            )));
        };

        let context = self.store.data_mut().push_resource(context)?;
        let context_rep = context.rep();

        let result = hook
            .call_async(&mut self.store, (context, definition, arguments, metadata))
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

        result?;

        Ok(())
    }

    /// Calls the pre authorize hook for a node
    pub async fn authorize_node_pre_execution(
        &mut self,
        context: SharedContextMap,
        definition: NodeDefinition,
        metadata: String,
    ) -> crate::Result<()> {
        let Some(hook) =
            self.get_hook::<NodePreParameters, NodePreResponse>(AUTHORIZE_NODE_PRE_EXECUTION_HOOK_FUNCTION)
        else {
            return Err(crate::Error::Internal(anyhow!(
                "authorize-node-pre-execution hook must be defined if using the @authorization directive"
            )));
        };

        let context = self.store.data_mut().push_resource(context)?;
        let context_rep = context.rep();

        let result = hook.call_async(&mut self.store, (context, definition, metadata)).await;

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

        result?;

        Ok(())
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

    /// A generic get hook we can use to find a different function from the interface.
    fn get_hook<I, O>(&mut self, function_name: &str) -> Option<TypedFunc<I, O>>
    where
        I: ComponentNamedList + Lower,
        O: ComponentNamedList + Lift,
    {
        let mut exports = self.instance.exports(&mut self.store);
        let mut root = exports.root();

        let Some(mut interface) = root.instance(COMPONENT_AUTHORIZATION) else {
            tracing::debug!(target: GRAFBASE_TARGET, "could not find export for authorization interface");
            return None;
        };

        match interface.typed_func(function_name) {
            Ok(hook) => {
                tracing::debug!(target: GRAFBASE_TARGET, "instantized the authorization hook WASM function");
                Some(hook)
            }
            Err(e) => {
                tracing::debug!(target: GRAFBASE_TARGET, "error instantizing the authorization hook WASM function: {e}");
                None
            }
        }
    }
}
