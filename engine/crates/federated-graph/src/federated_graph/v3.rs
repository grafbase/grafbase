pub use super::v2::{
    Definition, DirectiveId, Directives, Enum, EnumValue, FieldId, FieldProvides, FieldRequires, InputObject,
    InputValueDefinitions, InterfaceId, Key, ObjectId, Override, RootOperationTypes, Scalar, StringId, Subgraph,
    SubgraphId, Union, Value,
};

/// A composed federated graph.
///
/// ## API contract
///
/// Guarantees:
///
/// - All the identifiers are correct.
///
/// Does not guarantee:
///
/// - The ordering of items inside each `Vec`.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FederatedGraphV3 {
    pub subgraphs: Vec<Subgraph>,
    pub root_operation_types: RootOperationTypes,
    pub objects: Vec<Object>,
    pub interfaces: Vec<Interface>,
    pub fields: Vec<Field>,

    pub enums: Vec<Enum>,
    pub unions: Vec<Union>,
    pub scalars: Vec<Scalar>,
    pub input_objects: Vec<InputObject>,
    pub enum_values: Vec<EnumValue>,

    /// All [input value definitions](http://spec.graphql.org/October2021/#InputValueDefinition) in the federated graph. Concretely, these are arguments of output fields, and input object fields.
    pub input_value_definitions: Vec<InputValueDefinition>,

    /// All the strings in the federated graph, deduplicated.
    pub strings: Vec<String>,

    /// All composed directive instances (not definitions) in a federated graph.
    pub directives: Vec<Directive>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, PartialOrd)]
pub enum Directive {
    Authenticated,
    Deprecated {
        reason: Option<StringId>,
    },
    Inaccessible,
    Policy(Vec<Vec<String>>),
    RequiresScopes(Vec<Vec<StringId>>),

    Other {
        name: StringId,
        arguments: Vec<(StringId, Value)>,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Type {
    pub wrapping: u32,
    pub definition: Definition,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Field {
    pub name: StringId,
    pub field_type: Type,

    pub arguments: InputValueDefinitions,

    /// This is populated only of fields of entities. The Vec includes all subgraphs the field can
    /// be resolved in. For a regular field of an entity, it will be one subgraph, the subgraph
    /// where the entity field is defined. For a shareable field in an entity, this contains the
    /// subgraphs where the shareable field is defined on the entity. It may not be all the
    /// subgraphs.
    ///
    /// On fields of value types and input types, this is empty.
    pub resolvable_in: Vec<SubgraphId>,

    /// See [FieldProvides].
    pub provides: Vec<FieldProvides>,

    /// See [FieldRequires]
    pub requires: Vec<FieldRequires>,

    /// See [Override].
    pub overrides: Vec<Override>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Object {
    pub name: StringId,

    pub implements_interfaces: Vec<InterfaceId>,

    #[serde(rename = "resolvable_keys")]
    pub keys: Vec<Key>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Interface {
    pub name: StringId,

    pub implements_interfaces: Vec<InterfaceId>,

    /// All keys, for entity interfaces.
    #[serde(rename = "resolvable_keys")]
    pub keys: Vec<Key>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Directives,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<StringId>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InputValueDefinition {
    pub name: StringId,
    pub r#type: Type,
    pub directives: Directives,
    pub description: Option<StringId>,
}
