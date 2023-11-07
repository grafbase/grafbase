mod conversion;

/// This does NOT need to be backwards compatible. We'll probably cache it for performance, but it is not
/// the source of truth. If the cache is stale we would just re-create this Graph from its source:
/// federated_graph::FederatedGraph.
pub struct Graph {
    data_sources: Vec<DataSource>,
    subgraphs: Vec<Subgraph>,

    root_operation_types: RootOperationTypes,
    objects: Vec<Object>,
    // Sorted by object_id
    object_fields: Vec<ObjectField>,

    interfaces: Vec<Interface>,
    // Sorted by interface_id
    interface_fields: Vec<InterfaceField>,

    fields: Vec<Field>,

    enums: Vec<Enum>,
    unions: Vec<Union>,
    scalars: Vec<Scalar>,
    input_objects: Vec<InputObject>,

    /// All the strings in the supergraph, deduplicated.
    strings: Vec<String>,

    /// All the field types in the supergraph, deduplicated.
    field_types: Vec<FieldType>,
}

pub struct RootOperationTypes {
    pub query: ObjectId,
    pub mutation: Option<ObjectId>,
    pub subscription: Option<ObjectId>,
}

impl std::fmt::Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Graph>()).finish_non_exhaustive()
    }
}

pub enum DataSource {
    SubGraph(SubgraphId),
}

pub struct Subgraph {
    pub name: StringId,
    pub url: StringId,
}

pub struct Object {
    pub name: StringId,

    pub implements_interfaces: Vec<InterfaceId>,

    /// All _resolvable_ keys.
    pub resolvable_keys: Vec<Key>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

pub struct ObjectField {
    pub object_id: ObjectId,
    pub field_id: FieldId,
}

pub struct Key {
    /// The subgraph that can resolve the entity with these fields.
    pub subgraph_id: SubgraphId,

    /// Corresponds to the fields in an `@key` directive.
    pub fields: SelectionSet,
}

pub type SelectionSet = Vec<Selection>;

pub struct Selection {
    pub field: FieldId,
    pub subselection: SelectionSet,
}

pub struct Field {
    pub name: StringId,
    pub field_type_id: FieldTypeId,

    /// Includes one of:
    ///
    /// - One subgraph, where the field is defined, without directives.
    /// - One or more subgraphs where the field is shareable or part of the key.
    pub resolvable_in: Vec<DataSourceId>,

    /// See [FieldProvides].
    pub provides: Vec<FieldProvides>,

    /// See [FieldRequires]
    pub requires: Vec<FieldRequires>,

    pub arguments: Vec<FieldArgument>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

pub struct FieldArgument {
    pub name: StringId,
    pub type_id: FieldTypeId,
}

pub struct Directive {
    pub name: StringId,
    pub arguments: Vec<(StringId, Value)>,
}

pub enum Value {
    String(StringId),
    Int(i64),
    Float(StringId),
    Boolean(bool),
    EnumValue(StringId),
    Object(Vec<(StringId, Value)>),
    List(Vec<Value>),
}

pub enum Definition {
    Scalar(ScalarId),
    Object(ObjectId),
    Interface(InterfaceId),
    Union(UnionId),
    Enum(EnumId),
    InputObject(InputObjectId),
}

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

#[derive(Clone, Copy)]
pub enum ListWrapper {
    RequiredList,
    NullableList,
}

/// Represents an `@provides` directive on a field in a subgraph.
pub struct FieldProvides {
    pub data_source_id: DataSourceId,
    pub fields: SelectionSet,
}

/// Represents an `@requires` directive on a field in a subgraph.
pub struct FieldRequires {
    pub data_source_id: DataSourceId,
    pub fields: SelectionSet,
}

pub struct Interface {
    pub name: StringId,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

pub struct InterfaceField {
    pub interface_id: InterfaceId,
    pub field_id: FieldId,
}

pub struct Enum {
    pub name: StringId,
    pub values: Vec<EnumValue>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

pub struct EnumValue {
    pub value: StringId,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

pub struct Union {
    pub name: StringId,
    pub members: Vec<ObjectId>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

pub struct Scalar {
    pub name: StringId,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

pub struct InputObject {
    pub name: StringId,
    pub fields: Vec<InputObjectField>,

    /// All directives that made it through composition. Notably includes `@tag`.
    pub composed_directives: Vec<Directive>,
}

pub struct InputObjectField {
    pub name: StringId,
    pub field_type_id: FieldTypeId,
}

macro_rules! id_newtypes {
    ($($name:ident + $storage:ident + $out:ident,)*) => {
        $(
            #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
            pub struct $name(usize);

            impl std::ops::Index<$name> for Graph {
                type Output = $out;

                fn index(&self, index: $name) -> &$out {
                    &self.$storage[index.0]
                }
            }
        )*
    }
}

id_newtypes! {
    DataSourceId + data_sources + DataSource,
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

impl Graph {
    pub fn object_fields(&self, target: ObjectId) -> impl Iterator<Item = FieldId> + '_ {
        let start = self
            .object_fields
            .partition_point(|object_field| object_field.object_id < target);
        self.object_fields[start..].iter().map_while(move |object_field| {
            if object_field.object_id == target {
                Some(object_field.field_id)
            } else {
                None
            }
        })
    }

    pub fn interface_fields(&self, target: InterfaceId) -> impl Iterator<Item = FieldId> + '_ {
        let start = self
            .interface_fields
            .partition_point(|interface_field| interface_field.interface_id < target);
        self.interface_fields[start..].iter().map_while(move |interface_field| {
            if interface_field.interface_id == target {
                Some(interface_field.field_id)
            } else {
                None
            }
        })
    }

    pub fn query_fields(&self) -> impl Iterator<Item = FieldId> + '_ {
        self.object_fields(self.root_operation_types.query)
    }

    pub fn mutation_fields(&self) -> Box<dyn Iterator<Item = FieldId> + '_> {
        if let Some(mutation_object_id) = self.root_operation_types.mutation {
            Box::new(self.object_fields(mutation_object_id))
        } else {
            Box::new(std::iter::empty())
        }
    }
}
