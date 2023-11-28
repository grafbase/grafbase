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
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FederatedGraph {
    pub subgraphs: Vec<Subgraph>,

    pub root_operation_types: RootOperationTypes,
    pub objects: Vec<Object>,
    pub object_fields: Vec<ObjectField>,

    pub interfaces: Vec<Interface>,
    pub interface_fields: Vec<InterfaceField>,

    pub fields: Vec<Field>,

    pub enums: Vec<Enum>,
    pub unions: Vec<Union>,
    pub scalars: Vec<Scalar>,
    pub input_objects: Vec<InputObject>,

    /// All the strings in the supergraph, deduplicated.
    pub strings: Vec<String>,

    /// All the field types in the supergraph, deduplicated.
    pub field_types: Vec<FieldType>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RootOperationTypes {
    pub query: ObjectId,
    pub mutation: Option<ObjectId>,
    pub subscription: Option<ObjectId>,
}

impl std::fmt::Debug for FederatedGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<FederatedGraph>()).finish()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Subgraph {
    pub name: StringId,
    pub url: StringId,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Object {
    pub name: StringId,

    pub implements_interfaces: Vec<InterfaceId>,

    /// All _resolvable_ keys.
    pub resolvable_keys: Vec<Key>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ObjectField {
    pub object_id: ObjectId,
    pub field_id: FieldId,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Key {
    /// The subgraph that can resolve the entity with the fields in [Key::fields].
    pub subgraph_id: SubgraphId,

    /// Corresponds to the fields in an `@key` directive.
    pub fields: FieldSet,

    /// Correspond to the `@join__type(isInterfaceObject: true)` directive argument.
    pub is_interface_object: bool,
}

pub type FieldSet = Vec<FieldSetItem>;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FieldSetItem {
    pub field: FieldId,
    pub subselection: FieldSet,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub name: StringId,
    pub field_type_id: FieldTypeId,

    /// Includes the subgraph the field can be resolved in (= the subgraph that defines it), except
    /// where the field is shareable or part of the key, in which case `resolvable_in` will be
    /// `None`.
    pub resolvable_in: Option<SubgraphId>,

    /// See [FieldProvides].
    pub provides: Vec<FieldProvides>,

    /// See [FieldRequires]
    pub requires: Vec<FieldRequires>,

    pub arguments: Vec<FieldArgument>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FieldArgument {
    pub name: StringId,
    pub type_id: FieldTypeId,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Directive {
    pub name: StringId,
    pub arguments: Vec<(StringId, Value)>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum Value {
    String(StringId),
    Int(i64),
    Float(StringId),
    Boolean(bool),
    EnumValue(StringId),
    Object(Vec<(StringId, Value)>),
    List(Vec<Value>),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Definition {
    Scalar(ScalarId),
    Object(ObjectId),
    Interface(InterfaceId),
    Union(UnionId),
    Enum(EnumId),
    InputObject(InputObjectId),
}

#[derive(serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq)]
pub struct FieldType {
    pub kind: Definition,

    /// Is the innermost type required?
    ///
    /// Examples:
    ///
    /// - `String` => false
    /// - `String!` => true
    /// - `[String!]` => true
    /// - `[String]!` => false
    pub inner_is_required: bool,

    /// Innermost to outermost.
    pub list_wrappers: Vec<ListWrapper>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ListWrapper {
    RequiredList,
    NullableList,
}

/// Represents an `@provides` directive on a field in a subgraph.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FieldProvides {
    pub subgraph_id: SubgraphId,
    pub fields: FieldSet,
}

/// Represents an `@requires` directive on a field in a subgraph.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FieldRequires {
    pub subgraph_id: SubgraphId,
    pub fields: FieldSet,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Interface {
    pub name: StringId,

    pub implements_interfaces: Vec<InterfaceId>,

    /// All _resolvable_ keys, for entity interfaces.
    pub resolvable_keys: Vec<Key>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InterfaceField {
    pub interface_id: InterfaceId,
    pub field_id: FieldId,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Enum {
    pub name: StringId,
    pub values: Vec<EnumValue>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct EnumValue {
    pub value: StringId,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Union {
    pub name: StringId,
    pub members: Vec<ObjectId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Scalar {
    pub name: StringId,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InputObject {
    pub name: StringId,
    pub fields: Vec<InputObjectField>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InputObjectField {
    pub name: StringId,
    pub field_type_id: FieldTypeId,
}

macro_rules! id_newtypes {
    ($($name:ident + $storage:ident + $out:ident,)*) => {
        $(
            #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
            pub struct $name(pub usize);

            impl std::ops::Index<$name> for FederatedGraph {
                type Output = $out;

                fn index(&self, index: $name) -> &$out {
                    &self.$storage[index.0]
                }
            }

            impl std::ops::IndexMut<$name> for FederatedGraph {
                fn index_mut(&mut self, index: $name) -> &mut $out {
                    &mut self.$storage[index.0]
                }
            }
        )*
    }
}

id_newtypes! {
    EnumId + enums + Enum,
    FieldId + fields + Field,
    FieldTypeId + field_types + FieldType,
    InputObjectId + input_objects + InputObject,
    InterfaceId + interfaces + Interface,
    ObjectId + objects + Object,
    ScalarId + scalars + Scalar,
    StringId + strings + String,
    SubgraphId + subgraphs + Subgraph,
    UnionId + unions + Union,
}
