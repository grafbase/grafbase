use fxhash::FxBuildHasher;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

use crate::{SdkError, cbor, wit};

#[derive(Clone)]
pub(crate) struct IndexedSchema {
    name: String,
    directives: Vec<wit::Directive>,
    type_definitions: HashMap<DefinitionId, wit::TypeDefinition, FxBuildHasher>,
    field_definitions: HashMap<DefinitionId, wit::FieldDefinition, FxBuildHasher>,
    root_types: wit::RootTypes,
}

impl From<(String, wit::Schema)> for IndexedSchema {
    fn from((name, schema): (String, wit::Schema)) -> Self {
        Self {
            name,
            directives: schema.directives,
            type_definitions: schema
                .type_definitions
                .into_iter()
                .map(|def| {
                    let id = match &def {
                        wit::TypeDefinition::Scalar(scalar) => DefinitionId(scalar.id),
                        wit::TypeDefinition::Object(object) => DefinitionId(object.id),
                        wit::TypeDefinition::Interface(interface) => DefinitionId(interface.id),
                        wit::TypeDefinition::Union(union) => DefinitionId(union.id),
                        wit::TypeDefinition::Enum(enum_def) => DefinitionId(enum_def.id),
                        wit::TypeDefinition::InputObject(input_object) => DefinitionId(input_object.id),
                    };
                    (id, def)
                })
                .collect(),
            field_definitions: schema
                .field_definitions
                .into_iter()
                .map(|def| {
                    let id = DefinitionId(def.id);
                    (id, def)
                })
                .collect(),
            root_types: schema.root_types,
        }
    }
}

/// GraphQL schema
pub struct SubgraphSchema(pub(crate) IndexedSchema);

impl fmt::Debug for SubgraphSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubgraphSchema")
            .field("name", &self.subgraph_name())
            .field(
                "type_definitions",
                &format!("<{} type definitions>", self.type_definitions().len()),
            )
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}

impl SubgraphSchema {
    /// Name of the subgraph this schema belongs to
    pub fn subgraph_name(&self) -> &str {
        &self.0.name
    }

    /// Iterator over the definitions in this schema
    pub fn type_definitions(&self) -> impl ExactSizeIterator<Item = TypeDefinition<'_>> {
        let schema = &self.0;
        self.0.type_definitions.values().map(move |def| (schema, def).into())
    }

    /// Iterator over all object and interface fields in the schema
    pub fn iter_fields(&self) -> impl Iterator<Item = FieldDefinition<'_>> {
        let schema = &self.0;
        self.0
            .field_definitions
            .values()
            .map(|definition| FieldDefinition { schema, definition })
    }

    /// Retrieves a specific field definition by its unique identifier.
    pub fn field_definition(&self, id: DefinitionId) -> Option<FieldDefinition<'_>> {
        let schema = &self.0;
        self.0.field_definitions.get(&id).map(move |def| FieldDefinition {
            schema,
            definition: def,
        })
    }

    /// Retrieves a specific type definition by its unique identifier.
    pub fn type_definition(&self, id: DefinitionId) -> Option<TypeDefinition<'_>> {
        let schema = &self.0;
        self.0.type_definitions.get(&id).map(move |def| (schema, def).into())
    }

    /// Iterator over the directives applied to this schema
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'_>> {
        self.0.directives.iter().map(Into::into)
    }

    /// Query type id definition if any. Subgraph schema may only contain mutations or add fields
    /// to external objects.
    pub fn query(&self) -> Option<ObjectDefinition<'_>> {
        self.0.root_types.query_id.map(|id| {
            let Some(wit::TypeDefinition::Object(def)) = self.0.type_definitions.get(&DefinitionId(id)) else {
                unreachable!("Inconsitent schema");
            };
            (&self.0, def).into()
        })
    }

    /// Mutation type definition id if any
    pub fn mutation(&self) -> Option<ObjectDefinition<'_>> {
        self.0.root_types.mutation_id.map(|id| {
            let Some(wit::TypeDefinition::Object(def)) = self.0.type_definitions.get(&DefinitionId(id)) else {
                unreachable!("Inconsitent schema");
            };
            (&self.0, def).into()
        })
    }

    /// Subscription type definition id if any
    pub fn subscription(&self) -> Option<ObjectDefinition<'_>> {
        self.0.root_types.subscription_id.map(|id| {
            let Some(wit::TypeDefinition::Object(def)) = self.0.type_definitions.get(&DefinitionId(id)) else {
                unreachable!("Inconsitent schema");
            };
            (&self.0, def).into()
        })
    }
}

/// Identifier for a GraphQL definition within a schema
///
/// Provides a unique reference to different types of schema definitions such as
/// scalars, objects, interfaces, and other type definitions.
///
/// There is no particular guarantee on the nature of the u32, it could be a `u32::MAX`. It's only
/// ensured to be unique. It's recommended to use the `fxhash::FxHasher32` with a hashmap for best
/// performance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct DefinitionId(pub(crate) u32);

impl From<DefinitionId> for u32 {
    fn from(id: DefinitionId) -> u32 {
        id.0
    }
}

/// Enum representing the different types of GraphQL definitions
#[derive(Clone, Copy)]
pub enum TypeDefinition<'a> {
    /// A scalar type definition (e.g., String, Int, custom scalars)
    Scalar(ScalarDefinition<'a>),
    /// An object type definition
    Object(ObjectDefinition<'a>),
    /// An interface type definition
    Interface(InterfaceDefinition<'a>),
    /// A union type definition
    Union(UnionDefinition<'a>),
    /// An enum type definition
    Enum(EnumDefinition<'a>),
    /// An input object type definition
    InputObject(InputObjectDefinition<'a>),
}

impl fmt::Debug for TypeDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeDefinition::Scalar(def) => f.debug_tuple("Scalar").field(def).finish(),
            TypeDefinition::Object(def) => f.debug_tuple("Object").field(def).finish(),
            TypeDefinition::Interface(def) => f.debug_tuple("Interface").field(def).finish(),
            TypeDefinition::Union(def) => f.debug_tuple("Union").field(def).finish(),
            TypeDefinition::Enum(def) => f.debug_tuple("Enum").field(def).finish(),
            TypeDefinition::InputObject(def) => f.debug_tuple("InputObject").field(def).finish(),
        }
    }
}

impl std::fmt::Display for TypeDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a> TypeDefinition<'a> {
    /// Unique identifier for this type definition
    pub fn id(&self) -> DefinitionId {
        match self {
            TypeDefinition::Scalar(def) => def.id(),
            TypeDefinition::Object(def) => def.id(),
            TypeDefinition::Interface(def) => def.id(),
            TypeDefinition::Union(def) => def.id(),
            TypeDefinition::Enum(def) => def.id(),
            TypeDefinition::InputObject(def) => def.id(),
        }
    }

    /// Name of the type definition
    pub fn name(&self) -> &'a str {
        match self {
            TypeDefinition::Scalar(def) => def.name(),
            TypeDefinition::Object(def) => def.name(),
            TypeDefinition::Interface(def) => def.name(),
            TypeDefinition::Union(def) => def.name(),
            TypeDefinition::Enum(def) => def.name(),
            TypeDefinition::InputObject(def) => def.name(),
        }
    }
}

impl<'a> From<(&'a IndexedSchema, &'a wit::TypeDefinition)> for TypeDefinition<'a> {
    fn from((schema, definition): (&'a IndexedSchema, &'a wit::TypeDefinition)) -> Self {
        match definition {
            wit::TypeDefinition::Scalar(scalar) => TypeDefinition::Scalar((schema, scalar).into()),
            wit::TypeDefinition::Object(object) => TypeDefinition::Object((schema, object).into()),
            wit::TypeDefinition::Interface(interface) => TypeDefinition::Interface((schema, interface).into()),
            wit::TypeDefinition::Union(union) => TypeDefinition::Union((schema, union).into()),
            wit::TypeDefinition::Enum(enum_def) => TypeDefinition::Enum((schema, enum_def).into()),
            wit::TypeDefinition::InputObject(input_object) => {
                TypeDefinition::InputObject((schema, input_object).into())
            }
        }
    }
}

/// GraphQL scalar type definition
#[derive(Clone, Copy)]
pub struct ScalarDefinition<'a> {
    pub(crate) definition: &'a wit::ScalarDefinition,
}

impl fmt::Debug for ScalarDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScalarDefinition")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("specified_by_url", &self.specified_by_url())
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish()
    }
}

impl std::fmt::Display for ScalarDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a> From<(&'a IndexedSchema, &'a wit::ScalarDefinition)> for ScalarDefinition<'a> {
    fn from((_, definition): (&'a IndexedSchema, &'a wit::ScalarDefinition)) -> Self {
        Self { definition }
    }
}

impl<'a> ScalarDefinition<'a> {
    /// Unique identifier for this scalar definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.definition.id)
    }

    /// Name of the scalar type
    pub fn name(&self) -> &'a str {
        self.definition.name.as_str()
    }

    /// URL that specifies the behavior of this scalar, if any
    ///
    /// The specified by URL is used with custom scalars to point to
    /// a specification for how the scalar should be validated and parsed.
    pub fn specified_by_url(&self) -> Option<&'a str> {
        self.definition.specified_by_url.as_deref()
    }

    /// Iterator over the directives applied to this scalar
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.definition.directives.iter().map(Into::into)
    }
}

/// GraphQL object type definition
#[derive(Clone, Copy)]
pub struct ObjectDefinition<'a> {
    pub(crate) schema: &'a IndexedSchema,
    pub(crate) definition: &'a wit::ObjectDefinition,
}

impl fmt::Debug for ObjectDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectDefinition")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("fields", &self.fields().collect::<Vec<_>>())
            .field(
                "interfaces",
                &self.interfaces().map(|inf| inf.name()).collect::<Vec<_>>(),
            )
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish()
    }
}

impl std::fmt::Display for ObjectDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a> From<(&'a IndexedSchema, &'a wit::ObjectDefinition)> for ObjectDefinition<'a> {
    fn from((schema, definition): (&'a IndexedSchema, &'a wit::ObjectDefinition)) -> Self {
        Self { schema, definition }
    }
}

impl<'a> ObjectDefinition<'a> {
    /// Unique identifier for this object definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.definition.id)
    }

    /// Name of the object type
    pub fn name(&self) -> &'a str {
        self.definition.name.as_str()
    }

    /// Iterator over the fields defined in this object
    pub fn fields(&self) -> impl ExactSizeIterator<Item = FieldDefinition<'a>> + 'a {
        let schema = self.schema;
        self.definition.field_ids.iter().map(move |id| FieldDefinition {
            schema,
            definition: &schema.field_definitions[&DefinitionId(*id)],
        })
    }

    /// Iterator over the interfaces implemented by this object
    pub fn interfaces(&self) -> impl ExactSizeIterator<Item = InterfaceDefinition<'a>> + 'a {
        let schema = self.schema;
        self.definition.interface_ids.iter().map(move |&id| {
            let Some(wit::TypeDefinition::Interface(def)) = &schema.type_definitions.get(&DefinitionId(id)) else {
                unreachable!("Inconsitent schema");
            };
            (schema, def).into()
        })
    }

    /// Iterator over the directives applied to this object
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.definition.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL interface type definition
///
/// Interface types define a set of fields that multiple object types can implement.
/// Interfaces can also implement other interfaces.
#[derive(Clone, Copy)]
pub struct InterfaceDefinition<'a> {
    pub(crate) schema: &'a IndexedSchema,
    pub(crate) definition: &'a wit::InterfaceDefinition,
}

impl fmt::Debug for InterfaceDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterfaceDefinition")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("fields", &self.fields().collect::<Vec<_>>())
            .field(
                "interfaces",
                &self.interfaces().map(|inf| inf.name()).collect::<Vec<_>>(),
            )
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish()
    }
}

impl std::fmt::Display for InterfaceDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a> From<(&'a IndexedSchema, &'a wit::InterfaceDefinition)> for InterfaceDefinition<'a> {
    fn from((schema, definition): (&'a IndexedSchema, &'a wit::InterfaceDefinition)) -> Self {
        Self { schema, definition }
    }
}

impl<'a> InterfaceDefinition<'a> {
    /// Unique identifier for this interface definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.definition.id)
    }

    /// Name of the interface type
    pub fn name(&self) -> &'a str {
        self.definition.name.as_str()
    }

    /// Iterator over the fields defined in this interface
    pub fn fields(&self) -> impl ExactSizeIterator<Item = FieldDefinition<'a>> + 'a {
        let schema = self.schema;
        self.definition.field_ids.iter().map(move |id| FieldDefinition {
            definition: &schema.field_definitions[&DefinitionId(*id)],
            schema,
        })
    }

    /// Iterator over the interfaces implemented by this interface
    pub fn interfaces(&self) -> impl ExactSizeIterator<Item = InterfaceDefinition<'a>> + 'a {
        let schema = self.schema;
        self.definition.interface_ids.iter().map(move |&id| {
            let Some(wit::TypeDefinition::Interface(def)) = &schema.type_definitions.get(&DefinitionId(id)) else {
                unreachable!("Inconsitent schema");
            };
            (schema, def).into()
        })
    }

    /// Iterator over the directives applied to this interface
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.definition.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL entity definition, which can be either an object or an interface
/// It does not imply that this is a _federated_ entity with a `@key` directive.
#[derive(Clone, Copy)]
pub enum EntityDefinition<'a> {
    /// An object type definition
    Object(ObjectDefinition<'a>),
    /// An interface type definition
    Interface(InterfaceDefinition<'a>),
}

impl fmt::Debug for EntityDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityDefinition::Object(def) => f.debug_tuple("Object").field(def).finish(),
            EntityDefinition::Interface(def) => f.debug_tuple("Interface").field(def).finish(),
        }
    }
}

impl std::fmt::Display for EntityDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityDefinition::Object(def) => write!(f, "{}", def.name()),
            EntityDefinition::Interface(def) => write!(f, "{}", def.name()),
        }
    }
}

/// Represents a GraphQL union type definition
///
/// Union types define a type that could be one of several object types.
#[derive(Clone, Copy)]
pub struct UnionDefinition<'a> {
    pub(crate) schema: &'a IndexedSchema,
    pub(crate) definition: &'a wit::UnionDefinition,
}

impl fmt::Debug for UnionDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnionDefinition")
            .field("id", &self.id())
            .field("name", &self.name())
            .field(
                "member_types",
                &self.member_types().map(|obj| obj.name()).collect::<Vec<_>>(),
            )
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish()
    }
}

impl std::fmt::Display for UnionDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a> From<(&'a IndexedSchema, &'a wit::UnionDefinition)> for UnionDefinition<'a> {
    fn from((schema, definition): (&'a IndexedSchema, &'a wit::UnionDefinition)) -> Self {
        Self { schema, definition }
    }
}

impl<'a> UnionDefinition<'a> {
    /// Unique identifier for this union definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.definition.id)
    }

    /// Name of the union type
    pub fn name(&self) -> &'a str {
        self.definition.name.as_str()
    }

    /// Iterator over the member types that are part of this union
    pub fn member_types(&self) -> impl ExactSizeIterator<Item = ObjectDefinition<'a>> + 'a {
        let schema = self.schema;
        self.definition.member_types.iter().map(move |&id| {
            let Some(wit::TypeDefinition::Object(def)) = &schema.type_definitions.get(&DefinitionId(id)) else {
                unreachable!("Inconsitent schema");
            };
            (schema, def).into()
        })
    }

    /// Iterator over the directives applied to this union
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.definition.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL enum type definition
///
/// Enum types restrict a field to a finite set of values.
#[derive(Clone, Copy)]
pub struct EnumDefinition<'a> {
    pub(crate) definition: &'a wit::EnumDefinition,
}

impl fmt::Debug for EnumDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnumDefinition")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("values", &self.values().collect::<Vec<_>>())
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish()
    }
}

impl std::fmt::Display for EnumDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a> From<(&'a IndexedSchema, &'a wit::EnumDefinition)> for EnumDefinition<'a> {
    fn from((_, definition): (&'a IndexedSchema, &'a wit::EnumDefinition)) -> Self {
        Self { definition }
    }
}

impl<'a> EnumDefinition<'a> {
    /// Unique identifier for this enum definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.definition.id)
    }

    /// Name of the enum type
    pub fn name(&self) -> &'a str {
        self.definition.name.as_str()
    }

    /// Iterator over the possible values for this enum
    pub fn values(&self) -> impl ExactSizeIterator<Item = EnumValue<'a>> + 'a {
        self.definition.values.iter().map(Into::into)
    }

    /// Iterator over the directives applied to this enum
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.definition.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL input object type definition
///
/// Input objects are complex objects provided as arguments to fields,
/// consisting of a set of input fields.
#[derive(Clone, Copy)]
pub struct InputObjectDefinition<'a> {
    pub(crate) schema: &'a IndexedSchema,
    pub(crate) definition: &'a wit::InputObjectDefinition,
}

impl fmt::Debug for InputObjectDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputObjectDefinition")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("input_fields", &self.input_fields().collect::<Vec<_>>())
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish()
    }
}

impl std::fmt::Display for InputObjectDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a> From<(&'a IndexedSchema, &'a wit::InputObjectDefinition)> for InputObjectDefinition<'a> {
    fn from((schema, definition): (&'a IndexedSchema, &'a wit::InputObjectDefinition)) -> Self {
        Self { schema, definition }
    }
}

impl<'a> InputObjectDefinition<'a> {
    /// Unique identifier for this input object definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.definition.id)
    }

    /// Name of the input object type
    pub fn name(&self) -> &'a str {
        self.definition.name.as_str()
    }

    /// Iterator over the input fields defined in this input object
    pub fn input_fields(&self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> + 'a {
        self.definition.input_fields.iter().map(|field| InputValueDefinition {
            definition: field,
            schema: self.schema,
        })
    }

    /// Iterator over the directives applied to this input object
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.definition.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL field definition within an object or interface
///
/// Fields are the basic units of data in GraphQL. They define what data can be
/// fetched from a particular object or interface.
#[derive(Clone, Copy)]
pub struct FieldDefinition<'a> {
    pub(crate) schema: &'a IndexedSchema,
    pub(crate) definition: &'a wit::FieldDefinition,
}

impl fmt::Debug for FieldDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FieldDefinition")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("type", &self.ty())
            .field("arguments", &self.arguments().collect::<Vec<_>>())
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish()
    }
}

impl std::fmt::Display for FieldDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.parent_entity(), self.name())
    }
}

impl<'a> FieldDefinition<'a> {
    /// Unique identifier for this field definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.definition.id)
    }

    /// Name of the field
    pub fn name(&self) -> &'a str {
        self.definition.name.as_str()
    }

    /// Parent entity that this field belongs to
    pub fn parent_entity(&self) -> EntityDefinition<'a> {
        let def = &self.schema.type_definitions[&DefinitionId(self.definition.parent_type_id)];
        match def {
            wit::TypeDefinition::Object(obj) => EntityDefinition::Object((self.schema, obj).into()),
            wit::TypeDefinition::Interface(inf) => EntityDefinition::Interface((self.schema, inf).into()),
            _ => unreachable!("Field definition parent type must be an object or interface"),
        }
    }

    /// Type of value this field returns
    pub fn ty(&self) -> Type<'a> {
        (self.schema, &self.definition.ty).into()
    }

    /// Iterator over the arguments that can be passed to this field
    pub fn arguments(&self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> + 'a {
        self.definition.arguments.iter().map(|arg| InputValueDefinition {
            definition: arg,
            schema: self.schema,
        })
    }

    /// Iterator over the directives applied to this field
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.definition.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL type with its wrapping information
///
/// This struct contains information about a type's definition and any non-null
/// or list wrapping that may be applied to it.
#[derive(Clone, Copy)]
pub struct Type<'a> {
    schema: &'a IndexedSchema,
    ty: &'a wit::Ty,
}

impl fmt::Debug for Type<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Type")
            .field("definition", &self.definition().name())
            .field("wrapping", &self.wrapping().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> From<(&'a IndexedSchema, &'a wit::Ty)> for Type<'a> {
    fn from((schema, ty): (&'a IndexedSchema, &'a wit::Ty)) -> Self {
        Self { schema, ty }
    }
}

impl<'a> Type<'a> {
    /// Whether this type is non-null
    pub fn is_non_null(&self) -> bool {
        self.wrapping().last() == Some(WrappingType::NonNull)
    }

    /// Whether this type is a list
    pub fn is_list(&self) -> bool {
        self.wrapping().any(|w| matches!(w, WrappingType::List))
    }

    /// Iterator over the type wrappers applied to this type
    /// From the innermost to the outermost
    pub fn wrapping(&self) -> impl ExactSizeIterator<Item = WrappingType> + 'a {
        self.ty.wrapping.iter().map(|&w| w.into())
    }

    /// Identifier for the base type definition
    pub fn definition(&self) -> TypeDefinition<'a> {
        let Some(def) = self.schema.type_definitions.get(&DefinitionId(self.ty.definition_id)) else {
            unreachable!("Inconsitent schema");
        };
        (self.schema, def).into()
    }
}

/// Represents the different ways a GraphQL type can be wrapped
///
/// Types in GraphQL can be wrapped to indicate they are non-null or
/// represent a list of values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WrappingType {
    /// Indicates that the wrapped type cannot be null
    NonNull,
    /// Indicates that the wrapped type is a list of elements
    List,
}

impl From<wit::WrappingType> for WrappingType {
    fn from(wrapping: wit::WrappingType) -> Self {
        match wrapping {
            wit::WrappingType::NonNull => WrappingType::NonNull,
            wit::WrappingType::List => WrappingType::List,
        }
    }
}

/// Represents an input value definition in a GraphQL schema
///
/// Input values are used for arguments on fields and input object fields.
#[derive(Clone, Copy)]
pub struct InputValueDefinition<'a> {
    pub(crate) schema: &'a IndexedSchema,
    pub(crate) definition: &'a wit::InputValueDefinition,
}

impl fmt::Debug for InputValueDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputValueDefinition")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("type", &self.ty())
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> From<(&'a IndexedSchema, &'a wit::InputValueDefinition)> for InputValueDefinition<'a> {
    fn from((schema, definition): (&'a IndexedSchema, &'a wit::InputValueDefinition)) -> Self {
        Self { schema, definition }
    }
}

impl<'a> InputValueDefinition<'a> {
    /// Unique identifier for this input value definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.definition.id)
    }

    /// Name of the input value
    pub fn name(&self) -> &'a str {
        self.definition.name.as_str()
    }

    /// Type of this input value
    pub fn ty(&self) -> Type<'a> {
        (self.schema, &self.definition.ty).into()
    }

    /// Iterator over the directives applied to this input value
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.definition.directives.iter().map(Into::into)
    }
}

/// Represents a single possible value in a GraphQL enum definition
#[derive(Clone, Copy)]
pub struct EnumValue<'a>(&'a wit::EnumValue);

impl fmt::Debug for EnumValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnumValue")
            .field("name", &self.name())
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> From<&'a wit::EnumValue> for EnumValue<'a> {
    fn from(enum_value: &'a wit::EnumValue) -> Self {
        Self(enum_value)
    }
}

impl<'a> EnumValue<'a> {
    /// Name of this enum value
    pub fn name(&self) -> &'a str {
        self.0.name.as_str()
    }

    /// Iterator over the directives applied to this enum value
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL directive applied to a schema element
///
/// Directives provide a way to describe alternate runtime execution and type validation
/// behavior in a GraphQL document.
#[derive(Clone, Copy)]
pub struct Directive<'a>(pub(crate) DirectiveInner<'a>);

// TODO: write explicitly wit::Directive to use Cow instead of this enum.
#[derive(Clone, Copy)]
pub(crate) enum DirectiveInner<'a> {
    Wit(&'a wit::Directive),
    NameAndArgs { name: &'a str, arguments: &'a [u8] },
}

impl fmt::Debug for Directive<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Directive")
            .field("name", &self.name())
            .field("arguments", &"<binary arguments>")
            .finish()
    }
}

impl<'a> From<&'a wit::Directive> for Directive<'a> {
    fn from(directive: &'a wit::Directive) -> Self {
        Self(DirectiveInner::Wit(directive))
    }
}

impl<'a> Directive<'a> {
    /// Name of the directive
    pub fn name(&self) -> &'a str {
        match &self.0 {
            DirectiveInner::Wit(directive) => directive.name.as_str(),
            DirectiveInner::NameAndArgs { name, .. } => name,
        }
    }

    /// Deserializes the directive's arguments into the specified type.
    pub fn arguments<T>(&self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        cbor::from_slice::<T>(self.arguments_bytes()).map_err(Into::into)
    }

    /// Deserialize the arguments of the directive using a `DeserializeSeed`.
    #[inline]
    pub fn arguments_seed<T>(&self, seed: T) -> Result<T::Value, SdkError>
    where
        T: serde::de::DeserializeSeed<'a>,
    {
        cbor::from_slice_with_seed(self.arguments_bytes(), seed).map_err(Into::into)
    }

    fn arguments_bytes(&self) -> &'a [u8] {
        match &self.0 {
            DirectiveInner::Wit(directive) => &directive.arguments,
            DirectiveInner::NameAndArgs { arguments, .. } => arguments,
        }
    }
}
