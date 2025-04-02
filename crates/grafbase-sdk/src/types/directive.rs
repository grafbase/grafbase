use serde::Deserialize;

use crate::{cbor, wit, SdkError};

use super::FieldDefinitionDirectiveSite;

/// The directive and its arguments which define the extension in the GraphQL SDK.
pub struct SchemaDirective<'a> {
    pub(crate) subgraph_name: &'a str,
    pub(crate) directive: &'a wit::Directive,
}

impl<'a> SchemaDirective<'a> {
    /// The name of the directive.
    #[inline]
    pub fn name(&self) -> &'a str {
        &self.directive.name
    }

    /// The name of the subgraph this directive is part of.
    #[inline]
    pub fn subgraph_name(&self) -> &'a str {
        self.subgraph_name
    }

    /// The directive arguments. The output is a Serde structure, that must map to
    /// the arguments of the directive.
    ///
    /// Error is returned if the directive argument does not match the output structure.
    #[inline]
    pub fn arguments<T>(&'a self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        cbor::from_slice(&self.directive.arguments).map_err(Into::into)
    }
}

/// A field definition directive with its site information
pub struct FieldDefinitionDirective<'a>(&'a wit::FieldDefinitionDirective);

impl<'a> FieldDefinitionDirective<'a> {
    /// The name of the directive
    #[inline]
    pub fn name(&self) -> &'a str {
        &self.0.name
    }

    /// Arguments of the directive with any query data injected. Any argument that depends on
    /// response data will not be present here and be provided separately.
    pub fn arguments<T>(&self) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        minicbor_serde::from_slice(&self.0.arguments).map_err(Into::into)
    }

    /// Deserialize the arguments of the directive using a `DeserializeSeed`.
    pub fn deserialize_arguments_seed<T>(&self, seed: T) -> Result<T::Value, SdkError>
    where
        T: serde::de::DeserializeSeed<'a>,
    {
        let mut deserializer = minicbor_serde::Deserializer::new(&self.0.arguments);
        seed.deserialize(&mut deserializer).map_err(From::from)
    }

    ///Serialized arguments as sent by the host. There is no guarantee on the bytes.
    pub fn arguments_bytes(&self) -> &[u8] {
        &self.0.arguments
    }

    /// The site information for this directive
    #[inline]
    pub fn site(&self) -> FieldDefinitionDirectiveSite<'a> {
        (&self.0.site).into()
    }
}

impl<'a> From<&'a wit::FieldDefinitionDirective> for FieldDefinitionDirective<'a> {
    fn from(value: &'a wit::FieldDefinitionDirective) -> Self {
        Self(value)
    }
}
