use serde::Deserialize;
use std::fmt;

use crate::{SdkError, wit};

/// GraphQL schema
#[derive(Clone, Copy)]
pub struct SubgraphSchema<'a> {
    name: &'a str,
    schema: &'a wit::Schema,
}

impl fmt::Debug for SubgraphSchema<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubgraphSchema")
            .field("name", &self.name())
            .field(
                "type_definitions",
                &format!("<{} type definitions>", self.type_definitions().len()),
            )
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}

impl<'a> From<&'a (String, wit::Schema)> for SubgraphSchema<'a> {
    fn from((name, schema): &'a (String, wit::Schema)) -> Self {
        Self { name, schema }
    }
}

impl<'a> SubgraphSchema<'a> {
    /// Name of the subgraph this schema belongs to
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Iterator over the definitions in this schema
    pub fn type_definitions(&self) -> impl ExactSizeIterator<Item = TypeDefinition<'a>> + 'a {
        self.schema.type_definitions.iter().map(Into::into)
    }

    /// Iterator over the directives applied to this schema
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.schema.directives.iter().map(Into::into)
    }

    /// Query type id definition if any. Subgraph schema may only contain mutations or add fields
    /// to external objects.
    pub fn query_id(&self) -> Option<DefinitionId> {
        self.schema.root_types.query_id.map(DefinitionId)
    }

    /// Mutation type definition id if any
    pub fn mutation_id(&self) -> Option<DefinitionId> {
        self.schema.root_types.mutation_id.map(DefinitionId)
    }

    /// Subscription type definition id if any
    pub fn subscription_id(&self) -> Option<DefinitionId> {
        self.schema.root_types.subscription_id.map(DefinitionId)
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

impl<'a> From<&'a wit::TypeDefinition> for TypeDefinition<'a> {
    fn from(definition: &'a wit::TypeDefinition) -> Self {
        match definition {
            wit::TypeDefinition::Scalar(scalar) => TypeDefinition::Scalar(scalar.into()),
            wit::TypeDefinition::Object(object) => TypeDefinition::Object(object.into()),
            wit::TypeDefinition::Interface(interface) => TypeDefinition::Interface(interface.into()),
            wit::TypeDefinition::Union(union) => TypeDefinition::Union(union.into()),
            wit::TypeDefinition::Enum(enum_def) => TypeDefinition::Enum(enum_def.into()),
            wit::TypeDefinition::InputObject(input_object) => TypeDefinition::InputObject(input_object.into()),
        }
    }
}

/// GraphQL scalar type definition
#[derive(Clone, Copy)]
pub struct ScalarDefinition<'a>(&'a wit::ScalarDefinition);

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

impl<'a> From<&'a wit::ScalarDefinition> for ScalarDefinition<'a> {
    fn from(scalar: &'a wit::ScalarDefinition) -> Self {
        Self(scalar)
    }
}

impl<'a> ScalarDefinition<'a> {
    /// Unique identifier for this scalar definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.0.id)
    }

    /// Name of the scalar type
    pub fn name(&self) -> &'a str {
        self.0.name.as_str()
    }

    /// URL that specifies the behavior of this scalar, if any
    ///
    /// The specified by URL is used with custom scalars to point to
    /// a specification for how the scalar should be validated and parsed.
    pub fn specified_by_url(&self) -> Option<&'a str> {
        self.0.specified_by_url.as_deref()
    }

    /// Iterator over the directives applied to this scalar
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.0.directives.iter().map(Into::into)
    }
}

/// GraphQL object type definition
#[derive(Clone, Copy)]
pub struct ObjectDefinition<'a>(&'a wit::ObjectDefinition);

impl fmt::Debug for ObjectDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectDefinition")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("fields", &self.fields().collect::<Vec<_>>())
            .field("interfaces", &self.interfaces().collect::<Vec<_>>())
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> From<&'a wit::ObjectDefinition> for ObjectDefinition<'a> {
    fn from(object_definition: &'a wit::ObjectDefinition) -> Self {
        Self(object_definition)
    }
}

impl<'a> ObjectDefinition<'a> {
    /// Unique identifier for this object definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.0.id)
    }

    /// Name of the object type
    pub fn name(&self) -> &'a str {
        self.0.name.as_str()
    }

    /// Iterator over the fields defined in this object
    pub fn fields(&self) -> impl ExactSizeIterator<Item = FieldDefinition<'a>> + 'a {
        self.0.fields.iter().map(Into::into)
    }

    /// Iterator over the interfaces implemented by this object
    pub fn interfaces(&self) -> impl ExactSizeIterator<Item = DefinitionId> + 'a {
        self.0.interfaces.iter().map(|&id| DefinitionId(id))
    }

    /// Iterator over the directives applied to this object
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL interface type definition
///
/// Interface types define a set of fields that multiple object types can implement.
/// Interfaces can also implement other interfaces.
#[derive(Clone, Copy)]
pub struct InterfaceDefinition<'a>(&'a wit::InterfaceDefinition);

impl fmt::Debug for InterfaceDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterfaceDefinition")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("fields", &self.fields().collect::<Vec<_>>())
            .field("interfaces", &self.interfaces().collect::<Vec<_>>())
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> From<&'a wit::InterfaceDefinition> for InterfaceDefinition<'a> {
    fn from(interface: &'a wit::InterfaceDefinition) -> Self {
        Self(interface)
    }
}

impl<'a> InterfaceDefinition<'a> {
    /// Unique identifier for this interface definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.0.id)
    }

    /// Name of the interface type
    pub fn name(&self) -> &'a str {
        self.0.name.as_str()
    }

    /// Iterator over the fields defined in this interface
    pub fn fields(&self) -> impl ExactSizeIterator<Item = FieldDefinition<'a>> + 'a {
        self.0.fields.iter().map(Into::into)
    }

    /// Iterator over the interfaces implemented by this interface
    pub fn interfaces(&self) -> impl ExactSizeIterator<Item = DefinitionId> + 'a {
        self.0.interfaces.iter().map(|&id| DefinitionId(id))
    }

    /// Iterator over the directives applied to this interface
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL union type definition
///
/// Union types define a type that could be one of several object types.
#[derive(Clone, Copy)]
pub struct UnionDefinition<'a>(&'a wit::UnionDefinition);

impl fmt::Debug for UnionDefinition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnionDefinition")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("member_types", &self.member_types().collect::<Vec<_>>())
            .field("directives", &self.directives().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> From<&'a wit::UnionDefinition> for UnionDefinition<'a> {
    fn from(union: &'a wit::UnionDefinition) -> Self {
        Self(union)
    }
}

impl<'a> UnionDefinition<'a> {
    /// Unique identifier for this union definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.0.id)
    }

    /// Name of the union type
    pub fn name(&self) -> &'a str {
        self.0.name.as_str()
    }

    /// Iterator over the member types that are part of this union
    pub fn member_types(&self) -> impl ExactSizeIterator<Item = DefinitionId> + 'a {
        self.0.member_types.iter().map(|&id| DefinitionId(id))
    }

    /// Iterator over the directives applied to this union
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL enum type definition
///
/// Enum types restrict a field to a finite set of values.
#[derive(Clone, Copy)]
pub struct EnumDefinition<'a>(&'a wit::EnumDefinition);

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

impl<'a> From<&'a wit::EnumDefinition> for EnumDefinition<'a> {
    fn from(enum_def: &'a wit::EnumDefinition) -> Self {
        Self(enum_def)
    }
}

impl<'a> EnumDefinition<'a> {
    /// Unique identifier for this enum definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.0.id)
    }

    /// Name of the enum type
    pub fn name(&self) -> &'a str {
        self.0.name.as_str()
    }

    /// Iterator over the possible values for this enum
    pub fn values(&self) -> impl ExactSizeIterator<Item = EnumValue<'a>> + 'a {
        self.0.values.iter().map(Into::into)
    }

    /// Iterator over the directives applied to this enum
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL input object type definition
///
/// Input objects are complex objects provided as arguments to fields,
/// consisting of a set of input fields.
#[derive(Clone, Copy)]
pub struct InputObjectDefinition<'a>(&'a wit::InputObjectDefinition);

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

impl<'a> From<&'a wit::InputObjectDefinition> for InputObjectDefinition<'a> {
    fn from(input_object: &'a wit::InputObjectDefinition) -> Self {
        Self(input_object)
    }
}

impl<'a> InputObjectDefinition<'a> {
    /// Unique identifier for this input object definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.0.id)
    }

    /// Name of the input object type
    pub fn name(&self) -> &'a str {
        self.0.name.as_str()
    }

    /// Iterator over the input fields defined in this input object
    pub fn input_fields(&self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> + 'a {
        self.0.input_fields.iter().map(Into::into)
    }

    /// Iterator over the directives applied to this input object
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL field definition within an object or interface
///
/// Fields are the basic units of data in GraphQL. They define what data can be
/// fetched from a particular object or interface.
#[derive(Clone, Copy)]
pub struct FieldDefinition<'a>(&'a wit::FieldDefinition);

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

impl<'a> From<&'a wit::FieldDefinition> for FieldDefinition<'a> {
    fn from(field: &'a wit::FieldDefinition) -> Self {
        Self(field)
    }
}

impl<'a> FieldDefinition<'a> {
    /// Unique identifier for this field definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.0.id)
    }

    /// Name of the field
    pub fn name(&self) -> &'a str {
        self.0.name.as_str()
    }

    /// Type of value this field returns
    pub fn ty(&self) -> Type<'a> {
        (&self.0.ty).into()
    }

    /// Iterator over the arguments that can be passed to this field
    pub fn arguments(&self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> + 'a {
        self.0.arguments.iter().map(Into::into)
    }

    /// Iterator over the directives applied to this field
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL type with its wrapping information
///
/// This struct contains information about a type's definition and any non-null
/// or list wrapping that may be applied to it.
#[derive(Clone, Copy)]
pub struct Type<'a>(&'a wit::Ty);

impl fmt::Debug for Type<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Type")
            .field("definition_id", &self.definition_id())
            .field("wrapping", &self.wrapping().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> From<&'a wit::Ty> for Type<'a> {
    fn from(ty: &'a wit::Ty) -> Self {
        Self(ty)
    }
}

impl<'a> Type<'a> {
    /// Iterator over the type wrappers applied to this type
    /// From the innermost to the outermost
    pub fn wrapping(&self) -> impl ExactSizeIterator<Item = WrappingType> + 'a {
        self.0.wrapping.iter().map(|&w| w.into())
    }

    /// Identifier for the base type definition
    pub fn definition_id(&self) -> DefinitionId {
        DefinitionId(self.0.definition_id)
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
pub struct InputValueDefinition<'a>(&'a wit::InputValueDefinition);

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

impl<'a> From<&'a wit::InputValueDefinition> for InputValueDefinition<'a> {
    fn from(input_value: &'a wit::InputValueDefinition) -> Self {
        Self(input_value)
    }
}

impl<'a> InputValueDefinition<'a> {
    /// Unique identifier for this input value definition
    pub fn id(&self) -> DefinitionId {
        DefinitionId(self.0.id)
    }

    /// Name of the input value
    pub fn name(&self) -> &'a str {
        self.0.name.as_str()
    }

    /// Type of this input value
    pub fn ty(&self) -> Type<'a> {
        (&self.0.ty).into()
    }

    /// Iterator over the directives applied to this input value
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> + 'a {
        self.0.directives.iter().map(Into::into)
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
pub struct Directive<'a>(&'a wit::Directive);

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
        Self(directive)
    }
}

impl<'a> Directive<'a> {
    /// Name of the directive
    pub fn name(&self) -> &'a str {
        self.0.name.as_str()
    }

    /// Deserializes the directive's arguments into the specified type.
    pub fn arguments<T>(&self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        minicbor_serde::from_slice(&self.0.arguments).map_err(Into::into)
    }
}
