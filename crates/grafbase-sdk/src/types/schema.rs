use serde::Deserialize;

use crate::{SdkError, wit};

/// GraphQL schema
pub struct SubgraphSchema<'a> {
    name: &'a str,
    schema: &'a wit::Schema,
}

impl<'a> From<&'a (String, wit::Schema)> for SubgraphSchema<'a> {
    fn from((name, schema): &'a (String, wit::Schema)) -> Self {
        Self {
            name,
            schema
        }
    }
}

impl<'a> SubgraphSchema<'a> {
    /// Name of the subgraph this schema belongs to
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Iterator over the definitions in this schema
    pub fn definitions(&self) -> impl ExactSizeIterator<Item = Definition<'a>> {
        self.schema.definitions.iter().map(Into::into)
    }

    /// Iterator over the directives applied to this schema
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        self.schema.directives.iter().map(Into::into)
    }
}

/// Identifier for a GraphQL definition within a schema
///
/// Provides a unique reference to different types of schema definitions such as
/// scalars, objects, interfaces, and other type definitions.
pub struct DefinitionId(u32);

impl From<DefinitionId> for u32 {
    fn from(id: DefinitionId) -> u32 {
        id.0
    }
}

/// Enum representing the different types of GraphQL definitions
pub enum Definition<'a> {
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

impl<'a> From<&'a wit::Definition> for Definition<'a> {
    fn from(definition: &'a wit::Definition) -> Self {
        match definition {
            wit::Definition::Scalar(scalar) => Definition::Scalar(scalar.into()),
            wit::Definition::Object(object) => Definition::Object(object.into()),
            wit::Definition::Interface(interface) => Definition::Interface(interface.into()),
            wit::Definition::Union(union) => Definition::Union(union.into()),
            wit::Definition::Enum(enum_def) => Definition::Enum(enum_def.into()),
            wit::Definition::InputObject(input_object) => Definition::InputObject(input_object.into()),
        }
    }
}

/// GraphQL scalar type definition
pub struct ScalarDefinition<'a>(&'a wit::ScalarDefinition);

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
    pub fn name(&self) -> &str {
        self.0.name.as_str()
    }

    /// URL that specifies the behavior of this scalar, if any
    ///
    /// The specified by URL is used with custom scalars to point to
    /// a specification for how the scalar should be validated and parsed.
    pub fn specified_by_url(&self) -> Option<&str> {
        self.0.specified_by_url.as_deref()
    }

    /// Iterator over the directives applied to this scalar
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        self.0.directives.iter().map(Into::into)
    }
}

/// GraphQL object type definition
pub struct ObjectDefinition<'a>(&'a wit::ObjectDefinition);

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
    pub fn name(&self) -> &str {
        self.0.name.as_str()
    }

    /// Iterator over the fields defined in this object
    pub fn fields(&self) -> impl ExactSizeIterator<Item = FieldDefinition<'a>> {
        self.0.fields.iter().map(Into::into)
    }

    /// Iterator over the interfaces implemented by this object
    pub fn interfaces(&self) -> impl ExactSizeIterator<Item = DefinitionId> {
        self.0.interfaces.iter().map(|&id| DefinitionId(id))
    }

    /// Iterator over the directives applied to this object
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL interface type definition
///
/// Interface types define a set of fields that multiple object types can implement.
/// Interfaces can also implement other interfaces.
pub struct InterfaceDefinition<'a>(&'a wit::InterfaceDefinition);

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
    pub fn name(&self) -> &str {
        self.0.name.as_str()
    }

    /// Iterator over the fields defined in this interface
    pub fn fields(&self) -> impl ExactSizeIterator<Item = FieldDefinition<'a>> {
        self.0.fields.iter().map(Into::into)
    }

    /// Iterator over the interfaces implemented by this interface
    pub fn interfaces(&self) -> impl ExactSizeIterator<Item = DefinitionId> {
        self.0.interfaces.iter().map(|&id| DefinitionId(id))
    }

    /// Iterator over the directives applied to this interface
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL union type definition
///
/// Union types define a type that could be one of several object types.
pub struct UnionDefinition<'a>(&'a wit::UnionDefinition);

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
    pub fn name(&self) -> &str {
        self.0.name.as_str()
    }

    /// Iterator over the member types that are part of this union
    pub fn member_types(&self) -> impl ExactSizeIterator<Item = DefinitionId> {
        self.0.member_types.iter().map(|&id| DefinitionId(id))
    }

    /// Iterator over the directives applied to this union
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL enum type definition
///
/// Enum types restrict a field to a finite set of values.
pub struct EnumDefinition<'a>(&'a wit::EnumDefinition);

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
    pub fn name(&self) -> &str {
        self.0.name.as_str()
    }

    /// Iterator over the possible values for this enum
    pub fn values(&self) -> impl ExactSizeIterator<Item = EnumValue<'a>> {
        self.0.values.iter().map(Into::into)
    }

    /// Iterator over the directives applied to this enum
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL input object type definition
///
/// Input objects are complex objects provided as arguments to fields,
/// consisting of a set of input fields.
pub struct InputObjectDefinition<'a>(&'a wit::InputObjectDefinition);

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
    pub fn name(&self) -> &str {
        self.0.name.as_str()
    }

    /// Iterator over the input fields defined in this input object
    pub fn input_fields(&self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> {
        self.0.input_fields.iter().map(Into::into)
    }

    /// Iterator over the directives applied to this input object
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL field definition within an object or interface
///
/// Fields are the basic units of data in GraphQL. They define what data can be
/// fetched from a particular object or interface.
pub struct FieldDefinition<'a>(&'a wit::FieldDefinition);

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
    pub fn name(&self) -> &str {
        self.0.name.as_str()
    }

    /// Type of value this field returns
    pub fn ty(&self) -> Type<'a> {
        (&self.0.ty).into()
    }

    /// Iterator over the arguments that can be passed to this field
    pub fn arguments(&self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> {
        self.0.arguments.iter().map(Into::into)
    }

    /// Iterator over the directives applied to this field
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL type with its wrapping information
///
/// This struct contains information about a type's definition and any non-null
/// or list wrapping that may be applied to it.
pub struct Type<'a>(&'a wit::Ty);

impl<'a> From<&'a wit::Ty> for Type<'a> {
    fn from(ty: &'a wit::Ty) -> Self {
        Self(ty)
    }
}

impl Type<'_> {
    /// Iterator over the type wrappers applied to this type
    /// From the outermost to the innermost wrapper.
    pub fn wrapping(&self) -> impl ExactSizeIterator<Item = WrappingType> {
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
pub struct InputValueDefinition<'a>(&'a wit::InputValueDefinition);

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
    pub fn name(&self) -> &str {
        self.0.name.as_str()
    }

    /// Type of this input value
    pub fn ty(&self) -> Type<'a> {
        (&self.0.ty).into()
    }

    /// Iterator over the directives applied to this input value
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a single possible value in a GraphQL enum definition
pub struct EnumValue<'a>(&'a wit::EnumValue);

impl<'a> From<&'a wit::EnumValue> for EnumValue<'a> {
    fn from(enum_value: &'a wit::EnumValue) -> Self {
        Self(enum_value)
    }
}

impl<'a> EnumValue<'a> {
    /// Name of this enum value
    pub fn name(&self) -> &str {
        self.0.name.as_str()
    }

    /// Iterator over the directives applied to this enum value
    pub fn directives(&self) -> impl ExactSizeIterator<Item = Directive<'a>> {
        self.0.directives.iter().map(Into::into)
    }
}

/// Represents a GraphQL directive applied to a schema element
///
/// Directives provide a way to describe alternate runtime execution and type validation
/// behavior in a GraphQL document.
pub struct Directive<'a>(&'a wit::Directive);

impl<'a> From<&'a wit::Directive> for Directive<'a> {
    fn from(directive: &'a wit::Directive) -> Self {
        Self(directive)
    }
}

impl<'a> Directive<'a> {
    /// Name of the directive
    pub fn name(&self) -> &str {
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
