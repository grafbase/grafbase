use wasmtime::component::{ComponentType, Lower};

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
