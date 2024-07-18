use wasmtime::component::{ComponentType, Lower};

use crate::{
    context::SharedContextMap,
    names::{
        AUTHORIZE_EDGE_NODE_POST_EXECUTION_HOOK_FUNCTION, AUTHORIZE_EDGE_POST_EXECUTION_HOOK_FUNCTION,
        AUTHORIZE_EDGE_PRE_EXECUTION_HOOK_FUNCTION, AUTHORIZE_NODE_PRE_EXECUTION_HOOK_FUNCTION,
        AUTHORIZE_PARENT_EDGE_POST_EXECUTION_HOOK_FUNCTION, COMPONENT_AUTHORIZATION,
    },
    ComponentLoader, GuestResult,
};

use super::ComponentInstance;

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

/// The authorization hook is called if the requested type uses the authorization directive.
///
/// An instance of a function to be called from the Gateway level for the request.
/// The instance is meant to be separate for every request. The instance shares a memory space
/// with the guest, and cannot be shared with multiple requests.
pub struct AuthorizationHookInstance(ComponentInstance);

impl std::ops::Deref for AuthorizationHookInstance {
    type Target = ComponentInstance;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AuthorizationHookInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AuthorizationHookInstance {
    /// Creates a new instance of the authorization hook
    pub async fn new(loader: &ComponentLoader) -> crate::Result<Self> {
        ComponentInstance::new(loader, COMPONENT_AUTHORIZATION).await.map(Self)
    }

    /// Calls the pre authorize hook for an edge
    pub async fn authorize_edge_pre_execution(
        &mut self,
        context: SharedContextMap,
        definition: EdgeDefinition,
        arguments: String,
        metadata: String,
    ) -> crate::Result<()> {
        self.call3(
            AUTHORIZE_EDGE_PRE_EXECUTION_HOOK_FUNCTION,
            context,
            (definition, arguments, metadata),
        )
        .await?
        .map(|result: GuestResult<()>| result.map_err(Into::into))
        .ok_or_else(|| {
            crate::Error::from(format!(
                "{AUTHORIZE_EDGE_PRE_EXECUTION_HOOK_FUNCTION} hook must be defined if using the @authorized directive"
            ))
        })?
    }

    /// Calls the pre authorize hook for a node
    pub async fn authorize_node_pre_execution(
        &mut self,
        context: SharedContextMap,
        definition: NodeDefinition,
        metadata: String,
    ) -> crate::Result<()> {
        self.call2(
            AUTHORIZE_NODE_PRE_EXECUTION_HOOK_FUNCTION,
            context,
            (definition, metadata),
        )
        .await?
        .map(|result: GuestResult<()>| result.map_err(Into::into))
        .ok_or_else(|| {
            crate::Error::from(format!(
                "{AUTHORIZE_EDGE_PRE_EXECUTION_HOOK_FUNCTION} hook must be defined if using the @authorized directive"
            ))
        })?
    }

    /// Calls the post authorize hook for parent edge
    pub async fn authorize_parent_edge_post_execution(
        &mut self,
        context: SharedContextMap,
        definition: EdgeDefinition,
        parents: Vec<String>,
        metadata: String,
    ) -> crate::Result<Vec<Result<(), crate::GuestError>>> {
        self.call3(
            AUTHORIZE_PARENT_EDGE_POST_EXECUTION_HOOK_FUNCTION,
            context,
            (definition, parents, metadata),
        )
        .await?
        .map(|result: Vec<GuestResult<()>>| Ok(result))
        .ok_or_else(|| {
            crate::Error::from(format!(
                "{AUTHORIZE_PARENT_EDGE_POST_EXECUTION_HOOK_FUNCTION} hook must be defined if using the @authorized directive"
            ))
        })?
    }

    /// Calls the post authorize hook for parent edge
    pub async fn authorize_edge_node_post_execution(
        &mut self,
        context: SharedContextMap,
        definition: EdgeDefinition,
        nodes: Vec<String>,
        metadata: String,
    ) -> crate::Result<Vec<Result<(), crate::GuestError>>> {
        self.call3(
            AUTHORIZE_EDGE_NODE_POST_EXECUTION_HOOK_FUNCTION,
            context,
            (definition, nodes, metadata),
        )
        .await?
        .map(|result: Vec<GuestResult<()>>| Ok(result))
        .ok_or_else(|| {
            crate::Error::from(format!(
                "{AUTHORIZE_EDGE_NODE_POST_EXECUTION_HOOK_FUNCTION} hook must be defined if using the @authorized directive"
            ))
        })?
    }

    /// Calls the post authorize hook for parent edge
    pub async fn authorize_edge_post_execution(
        &mut self,
        context: SharedContextMap,
        definition: EdgeDefinition,
        edges: Vec<(String, Vec<String>)>,
        metadata: String,
    ) -> crate::Result<Vec<Result<(), crate::GuestError>>> {
        self.call3(
            AUTHORIZE_EDGE_POST_EXECUTION_HOOK_FUNCTION,
            context,
            (definition, edges, metadata),
        )
        .await?
        .map(|result: Vec<GuestResult<()>>| Ok(result))
        .ok_or_else(|| {
            crate::Error::from(format!(
                "{AUTHORIZE_EDGE_POST_EXECUTION_HOOK_FUNCTION} hook must be defined if using the @authorized directive"
            ))
        })?
    }
}
