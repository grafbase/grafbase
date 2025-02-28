use serde::Deserialize;

use crate::{wit, SdkError};

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
}

impl<'a> From<&'a wit::DirectiveSite> for DirectiveSite<'a> {
    fn from(value: &'a wit::DirectiveSite) -> Self {
        match value {
            wit::DirectiveSite::Object(site) => DirectiveSite::Object(site.into()),
            wit::DirectiveSite::FieldDefinition(site) => DirectiveSite::FieldDefinition(site.into()),
            wit::DirectiveSite::Interface(site) => DirectiveSite::Interface(site.into()),
            wit::DirectiveSite::Union(site) => DirectiveSite::Union(site.into()),
        }
    }
}

impl<'a> DirectiveSite<'a> {
    /// Arguments of the directive with any query data injected. Any argument that depends on
    /// response data will not be present here and be provided separately.
    pub fn arguments<T>(&self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        minicbor_serde::from_slice(match self {
            DirectiveSite::Object(site) => &site.0.arguments,
            DirectiveSite::FieldDefinition(site) => &site.0.arguments,
            DirectiveSite::Interface(site) => &site.0.arguments,
            DirectiveSite::Union(site) => &site.0.arguments,
        })
        .map_err(Into::into)
    }
}

/// A directive site for object types
pub struct ObjectDirectiveSite<'a>(&'a wit::ObjectDirectiveSite);

impl<'a> ObjectDirectiveSite<'a> {
    /// The name of the object type
    #[inline]
    pub fn object_name(&self) -> &str {
        &self.0.object_name
    }

    /// Arguments of the directive with any query data injected. Any argument that depends on
    /// response data will not be present here and be provided separately.
    pub fn arguments<T>(&self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        minicbor_serde::from_slice(&self.0.arguments).map_err(Into::into)
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
    pub fn parent_type_name(&self) -> &str {
        &self.0.parent_type_name
    }

    /// The name of the field
    #[inline]
    pub fn field_name(&self) -> &str {
        &self.0.field_name
    }

    /// Arguments of the directive with any query data injected. Any argument that depends on
    /// response data will not be present here and be provided separately.
    pub fn arguments<T>(&self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        minicbor_serde::from_slice(&self.0.arguments).map_err(Into::into)
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
    pub fn union_name(&self) -> &str {
        &self.0.union_name
    }

    /// Arguments of the directive with any query data injected. Any argument that depends on
    /// response data will not be present here and be provided separately.
    pub fn arguments<T>(&self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        minicbor_serde::from_slice(&self.0.arguments).map_err(Into::into)
    }
}

impl<'a> From<&'a wit::UnionDirectiveSite> for UnionDirectiveSite<'a> {
    fn from(value: &'a wit::UnionDirectiveSite) -> Self {
        Self(value)
    }
}

/// A directive site for interface types
pub struct InterfaceDirectiveSite<'a>(&'a wit::InterfaceDirectiveSite);

impl<'a> InterfaceDirectiveSite<'a> {
    /// The name of the interface type
    #[inline]
    pub fn interface_name(&self) -> &str {
        &self.0.interface_name
    }

    /// Arguments of the directive with any query data injected. Any argument that depends on
    /// response data will not be present here and be provided separately.
    pub fn arguments<T>(&self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        minicbor_serde::from_slice(&self.0.arguments).map_err(Into::into)
    }
}

impl<'a> From<&'a wit::InterfaceDirectiveSite> for InterfaceDirectiveSite<'a> {
    fn from(value: &'a wit::InterfaceDirectiveSite) -> Self {
        Self(value)
    }
}
