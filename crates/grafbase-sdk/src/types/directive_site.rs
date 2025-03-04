use crate::wit;

/// The site where a directive is applied in the GraphQL schema.
pub enum DirectiveSite<'a> {
    /// Directive applied to an object type
    Object(ObjectDirectiveSite<'a>),
    /// Directive applied to a field definition
    FieldDefinition(FieldDefinitionDirectiveSite<'a>),
    /// Directive applied to an interface
    Interface(InterfaceDirectiveSite<'a>),
    /// Directive applied to a union
    Union(UnionDirectiveSite<'a>),
    /// Directive applied to an enum
    Enum(EnumDirectiveSite<'a>),
    /// Directive applied to a scalar
    Scalar(ScalarDirectiveSite<'a>),
}

impl<'a> From<&'a wit::DirectiveSite> for DirectiveSite<'a> {
    fn from(value: &'a wit::DirectiveSite) -> Self {
        match value {
            wit::DirectiveSite::Object(site) => DirectiveSite::Object(site.into()),
            wit::DirectiveSite::FieldDefinition(site) => DirectiveSite::FieldDefinition(site.into()),
            wit::DirectiveSite::Interface(site) => DirectiveSite::Interface(site.into()),
            wit::DirectiveSite::Union(site) => DirectiveSite::Union(site.into()),
            wit::DirectiveSite::Enum(site) => DirectiveSite::Enum(site.into()),
            wit::DirectiveSite::Scalar(site) => DirectiveSite::Scalar(site.into()),
        }
    }
}

/// A directive site for object types
pub struct ObjectDirectiveSite<'a>(&'a wit::ObjectDirectiveSite);

impl<'a> ObjectDirectiveSite<'a> {
    /// The name of the object type
    #[inline]
    pub fn object_name(&self) -> &'a str {
        &self.0.object_name
    }
}

impl<'a> From<&'a wit::ObjectDirectiveSite> for ObjectDirectiveSite<'a> {
    fn from(value: &'a wit::ObjectDirectiveSite) -> Self {
        Self(value)
    }
}

/// A directive site for field definitions
pub struct FieldDefinitionDirectiveSite<'a>(&'a wit::FieldDefinitionDirectiveSite);

impl<'a> FieldDefinitionDirectiveSite<'a> {
    /// The name of the parent type containing this field
    #[inline]
    pub fn parent_type_name(&self) -> &'a str {
        &self.0.parent_type_name
    }

    /// The name of the field
    #[inline]
    pub fn field_name(&self) -> &'a str {
        &self.0.field_name
    }
}

impl<'a> From<&'a wit::FieldDefinitionDirectiveSite> for FieldDefinitionDirectiveSite<'a> {
    fn from(value: &'a wit::FieldDefinitionDirectiveSite) -> Self {
        Self(value)
    }
}

/// A directive site for union types
pub struct UnionDirectiveSite<'a>(&'a wit::UnionDirectiveSite);

impl<'a> UnionDirectiveSite<'a> {
    /// The name of the union type
    #[inline]
    pub fn union_name(&self) -> &'a str {
        &self.0.union_name
    }
}

impl<'a> From<&'a wit::UnionDirectiveSite> for UnionDirectiveSite<'a> {
    fn from(value: &'a wit::UnionDirectiveSite) -> Self {
        Self(value)
    }
}

/// A directive site for interfaces
pub struct InterfaceDirectiveSite<'a>(&'a wit::InterfaceDirectiveSite);

impl<'a> InterfaceDirectiveSite<'a> {
    /// The name of the interface type
    #[inline]
    pub fn interface_name(&self) -> &'a str {
        &self.0.interface_name
    }
}

impl<'a> From<&'a wit::InterfaceDirectiveSite> for InterfaceDirectiveSite<'a> {
    fn from(value: &'a wit::InterfaceDirectiveSite) -> Self {
        Self(value)
    }
}

/// A directive site for scalars
pub struct ScalarDirectiveSite<'a>(&'a wit::ScalarDirectiveSite);

impl<'a> ScalarDirectiveSite<'a> {
    /// The name of the scalar type
    #[inline]
    pub fn scalar_name(&self) -> &'a str {
        &self.0.scalar_name
    }
}

impl<'a> From<&'a wit::ScalarDirectiveSite> for ScalarDirectiveSite<'a> {
    fn from(value: &'a wit::ScalarDirectiveSite) -> Self {
        Self(value)
    }
}

/// A directive site for enums
pub struct EnumDirectiveSite<'a>(&'a wit::EnumDirectiveSite);

impl<'a> EnumDirectiveSite<'a> {
    /// The name of the enum type
    #[inline]
    pub fn enum_name(&self) -> &'a str {
        &self.0.enum_name
    }
}

impl<'a> From<&'a wit::EnumDirectiveSite> for EnumDirectiveSite<'a> {
    fn from(value: &'a wit::EnumDirectiveSite) -> Self {
        Self(value)
    }
}
