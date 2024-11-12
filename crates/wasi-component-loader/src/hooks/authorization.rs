use wasmtime::component::{ComponentType, Lower};

use crate::{
    context::SharedContext,
    names::{
        AUTHORIZATION_INTERFACE, AUTHORIZE_EDGE_NODE_POST_EXECUTION_HOOK_FUNCTION,
        AUTHORIZE_EDGE_POST_EXECUTION_HOOK_FUNCTION, AUTHORIZE_EDGE_PRE_EXECUTION_HOOK_FUNCTION,
        AUTHORIZE_NODE_PRE_EXECUTION_HOOK_FUNCTION, AUTHORIZE_PARENT_EDGE_POST_EXECUTION_HOOK_FUNCTION,
    },
    ComponentLoader, GuestResult,
};

use super::{component_instance, ComponentInstance};

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

component_instance!(AuthorizationComponentInstance: AUTHORIZATION_INTERFACE);

impl AuthorizationComponentInstance {
    /// Calls the pre authorize hook for an edge.
    ///
    /// This function is invoked before the execution of an edge operation. It checks
    /// whether the operation is authorized based on the provided `definition`, `arguments`,
    /// and `metadata`. If the authorization check fails, an error is returned.
    ///
    /// # Arguments
    ///
    /// - `context`: The shared context for the operation.
    /// - `definition`: The edge definition containing type and field names.
    /// - `arguments`: A string representing the arguments for the operation.
    /// - `metadata`: A string containing metadata for the operation.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure of the authorization check.
    pub async fn authorize_edge_pre_execution(
        &mut self,
        context: SharedContext,
        definition: EdgeDefinition,
        arguments: String,
        metadata: String,
    ) -> crate::Result<()> {
        self.call3_one_output(
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

    /// Calls the pre authorize hook for a node.
    ///
    /// This function is invoked before the execution of a node operation. It checks
    /// whether the operation is authorized based on the provided `definition` and
    /// `metadata`. If the authorization check fails, an error is returned.
    ///
    /// # Arguments
    ///
    /// - `context`: The shared context for the operation.
    /// - `definition`: The node definition containing the type name.
    /// - `metadata`: A string containing metadata for the operation.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure of the authorization check.
    pub async fn authorize_node_pre_execution(
        &mut self,
        context: SharedContext,
        definition: NodeDefinition,
        metadata: String,
    ) -> crate::Result<()> {
        self.call2_one_output(
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

    /// Calls the post authorize hook for a parent edge.
    ///
    /// This function is invoked after the execution of a parent edge operation. It checks
    /// whether the operation is authorized based on the provided `definition`, `parents`,
    /// and `metadata`. If the authorization check fails, an error is returned.
    ///
    /// # Arguments
    ///
    /// - `context`: The shared context for the operation.
    /// - `definition`: The edge definition containing type and field names.
    /// - `parents`: A vector of strings representing the parent nodes for the operation.
    /// - `metadata`: A string containing metadata for the operation.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of results indicating success or failure
    /// of the authorization checks for each parent node.
    pub async fn authorize_parent_edge_post_execution(
        &mut self,
        context: SharedContext,
        definition: EdgeDefinition,
        parents: Vec<String>,
        metadata: String,
    ) -> crate::Result<Vec<Result<(), crate::GuestError>>> {
        self.call3_one_output(
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

    /// Calls the post authorize hook for an edge involving nodes.
    ///
    /// This function is invoked after the execution of an edge operation involving
    /// nodes. It checks whether the operation is authorized based on the provided
    /// `definition`, `nodes`, and `metadata`. If the authorization check fails,
    /// an error is returned.
    ///
    /// # Parameters
    ///
    /// - `context`: The shared context for the operation.
    /// - `definition`: The edge definition containing type and field names.
    /// - `nodes`: A vector of strings representing the nodes for the operation.
    /// - `metadata`: A string containing metadata for the operation.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of results indicating success or
    /// failure of the authorization checks for each node.
    pub async fn authorize_edge_node_post_execution(
        &mut self,
        context: SharedContext,
        definition: EdgeDefinition,
        nodes: Vec<String>,
        metadata: String,
    ) -> crate::Result<Vec<Result<(), crate::GuestError>>> {
        self.call3_one_output(
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

    /// Calls the post authorize hook for an edge.
    ///
    /// This function is invoked after the execution of an edge operation. It checks
    /// whether the operation is authorized based on the provided `definition`, `edges`,
    /// and `metadata`. If the authorization check fails, an error is returned.
    ///
    /// # Arguments
    ///
    /// - `context`: The shared context for the operation.
    /// - `definition`: The edge definition containing type and field names.
    /// - `edges`: A vector of tuples where each tuple contains a string representing an edge
    ///   and a vector of strings representing associated nodes for the operation.
    /// - `metadata`: A string containing metadata for the operation.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of results indicating success or failure
    /// of the authorization checks for each edge.
    pub async fn authorize_edge_post_execution(
        &mut self,
        context: SharedContext,
        definition: EdgeDefinition,
        edges: Vec<(String, Vec<String>)>,
        metadata: String,
    ) -> crate::Result<Vec<Result<(), crate::GuestError>>> {
        self.call3_one_output(
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
