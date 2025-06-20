use std::borrow::Cow;

use serde::{Deserialize, de::DeserializeSeed};

use crate::{
    SdkError,
    types::{ArgumentsId, DefinitionId, Directive, Field, FieldDefinition, SelectionSet, SubgraphSchema, Variables},
    wit,
};

/// Represents a resolved field in the context of a subgraph and its parent type
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ResolvedField<'a> {
    pub(crate) subgraph_name: &'a str,
    pub(crate) directive_name: &'a str,
    pub(crate) directive_arguments: &'a [u8],
    pub(crate) fields: Cow<'a, [wit::Field]>,
    pub(crate) root_field_ix: usize,
}

impl<'a> TryFrom<&'a [u8]> for ResolvedField<'a> {
    type Error = SdkError;
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        postcard::from_bytes(value).map_err(|err| format!("Failed to deserialize field data: {err}").into())
    }
}

impl From<ResolvedField<'_>> for Vec<u8> {
    fn from(value: ResolvedField<'_>) -> Self {
        postcard::to_stdvec(&value).expect("Failed to serialize ResolvedField")
    }
}

impl<'a> ResolvedField<'a> {
    /// Reference to the root field
    pub fn as_ref(&self) -> Field<'_> {
        Field {
            fields: &self.fields,
            field: &self.fields[self.root_field_ix],
        }
    }

    /// Returns the name of the subgraph this field belongs to.
    pub fn subgraph_name(&self) -> &'a str {
        self.subgraph_name
    }

    /// Gets the arguments ID of this field, if any
    pub fn arguments_id(&self) -> Option<ArgumentsId> {
        self.as_ref().arguments_id()
    }

    /// Definition of the field within the subgraph schema.
    pub fn definition<'s>(&self, schema: &'s SubgraphSchema) -> FieldDefinition<'s> {
        schema
            .field_definition(self.definition_id())
            .expect("Field definition not found, the wrong subgraph may have been used.")
    }

    /// Field definition id.
    pub fn definition_id(&self) -> DefinitionId {
        self.as_ref().definition_id()
    }

    /// Deserializes the arguments of this field into the specified type
    pub fn arguments<'de, T>(&self, variables: &'de Variables) -> Result<T, SdkError>
    where
        T: Deserialize<'de>,
    {
        self.as_ref().arguments(variables)
    }

    /// Deserializes the arguments of this field into the specified type with the given seed.
    pub fn arguments_seed<'de, Seed>(&self, variables: &'de Variables, seed: Seed) -> Result<Seed::Value, SdkError>
    where
        Seed: DeserializeSeed<'de>,
    {
        self.as_ref().arguments_seed(variables, seed)
    }

    /// Gets the selection set of this field
    pub fn selection_set(&self) -> SelectionSet<'_> {
        self.as_ref().selection_set()
    }

    /// Returns the resolver directive associated with this field
    pub fn directive(&self) -> Directive<'a> {
        Directive(super::DirectiveInner::NameAndArgs {
            name: self.directive_name,
            arguments: self.directive_arguments,
        })
    }
}
