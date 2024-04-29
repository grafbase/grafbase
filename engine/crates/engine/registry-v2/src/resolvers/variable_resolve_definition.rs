#![allow(deprecated)]
use std::borrow::Cow;

/// Describe what should be done by the GraphQL Server to resolve this Variable.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum VariableResolveDefinition {
    /// A Debug VariableResolveDefinition where you can just put the Value you
    /// would like to have.
    DebugString(Cow<'static, str>),
    /// Check the last Resolver in the Query Graph and try to resolve the
    /// variable defined in this field.
    InputTypeName(Cow<'static, str>),
    /// Check the last Resolver in the Query Graph, try to resolve the
    /// variable defined in this field and then apply connector transforms
    ConnectorInputTypeName(Cow<'static, str>),
    /// Resolve a Value by querying the ResolverContextData with a key_id.
    /// What is store in the ResolverContextData is described on each Resolver
    /// implementation.
    #[deprecated = "Should not use Context anymore in SDL def"]
    ResolverData(Cow<'static, str>),
    /// Resolve a Value by querying the most recent ancestor resolver property.
    LocalData(Cow<'static, str>),
    /// Resolve a Value of a specific type by querying the most recent ancestor resolver property
    ///
    /// This particular branch expects the data to come from an external source and will
    /// apply the transforms associated with the InputValueType to that data.
    LocalDataWithTransforms(Box<(Cow<'static, str>, String)>),
}

impl VariableResolveDefinition {
    pub fn connector_input_type_name(value: impl Into<Cow<'static, str>>) -> Self {
        Self::ConnectorInputTypeName(value.into())
    }

    pub fn local_data(value: impl Into<Cow<'static, str>>) -> Self {
        Self::LocalData(value.into())
    }

    pub fn local_data_with_transforms(value: impl Into<Cow<'static, str>>, ty: String) -> Self {
        Self::LocalDataWithTransforms(Box::new((value.into(), ty)))
    }
}
