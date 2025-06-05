use crate::{SdkError, wit};
use serde::{Deserialize, de::DeserializeSeed};

use super::DefinitionId;

/// A field within a GraphQL query
#[derive(Clone, Copy)]
pub struct Field<'a> {
    pub(crate) fields: &'a [wit::Field],
    pub(crate) field: &'a wit::Field,
}

impl<'a> Field<'a> {
    /// Gets the alias of this field, if any
    pub fn alias(&self) -> Option<&'a str> {
        self.field.alias.as_deref()
    }

    /// Gets the arguments ID of this field, if any
    pub fn arguments_id(&self) -> Option<ArgumentsId> {
        self.field.arguments.map(ArgumentsId)
    }

    /// Field definition id.
    pub fn definition_id(&self) -> DefinitionId {
        DefinitionId(self.field.definition_id)
    }

    /// Deserializes the arguments of this field into the specified type
    pub fn arguments<'de, T>(&self, variables: &'de Variables) -> Result<T, SdkError>
    where
        T: Deserialize<'de>,
    {
        match self.field.arguments {
            Some(id) => variables.get(id.into()),
            None => Ok(T::deserialize(serde_json::json!({}))?),
        }
    }

    /// Deserializes the arguments of this field into the specified type with the given seed.
    pub fn arguments_seed<'de, Seed>(&self, variables: &'de Variables, seed: Seed) -> Result<Seed::Value, SdkError>
    where
        Seed: DeserializeSeed<'de>,
    {
        match self.field.arguments {
            Some(id) => variables.get_seed(id.into(), seed),
            None => Ok(seed.deserialize(serde_json::json!({}))?),
        }
    }

    /// Gets the selection set of this field
    pub fn selection_set(&self) -> SelectionSet<'a> {
        self.field
            .selection_set
            .map(|selection_set| SelectionSet {
                fields: self.fields,
                selection_set,
            })
            .unwrap_or_else(|| SelectionSet {
                fields: &[],
                selection_set: wit::SelectionSet {
                    fields_ordered_by_parent_entity: (0, 0),
                    requires_typename: false,
                },
            })
    }
}

/// Represents a selection set in a GraphQL query
///
/// A selection set is a group of fields selected together in a query.
#[derive(Clone, Copy)]
pub struct SelectionSet<'a> {
    fields: &'a [wit::Field],
    selection_set: wit::SelectionSet,
}

impl<'a> SelectionSet<'a> {
    /// Whether this selection set is empty. Can only happen for scalars and enums.
    pub fn is_empty(&self) -> bool {
        self.fields().len() == 0
    }

    /// Iterator of the fields of this selection set. For best performance, you should respect the
    /// field ordering in the resolver data.
    pub fn fields(&self) -> impl ExactSizeIterator<Item = Field<'a>> + 'a {
        self.fields_ordered_by_parent_entity()
    }

    /// Iterator over the fields in this selection set, ordered by their parent entity. However, how parent
    /// entities are ordered (by id, name, etc.) is undefined. For best performance, you should respect the
    /// field ordering in the resolver data.
    pub fn fields_ordered_by_parent_entity(&self) -> impl ExactSizeIterator<Item = Field<'a>> + 'a {
        let (start, end) = self.selection_set.fields_ordered_by_parent_entity;
        let fields = self.fields;
        fields[usize::from(start)..usize::from(end)]
            .iter()
            .map(move |field| Field { fields, field })
    }

    /// Whether this selection set requires a `__typename` field
    /// The Gateway doesn't need the typename for objects and for various simple cases. But if
    /// multiple type conditions are applied, it'll be required.
    pub fn requires_typename(&self) -> bool {
        self.selection_set.requires_typename
    }
}

/// Identifier for arguments in a GraphQL query
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArgumentsId(wit::ArgumentsId);

impl From<wit::ArgumentsId> for ArgumentsId {
    fn from(id: wit::ArgumentsId) -> Self {
        Self(id)
    }
}

/// All argument values for a given selection set, to be used with [Field].
pub struct Variables(Vec<(wit::ArgumentsId, Vec<u8>)>);

impl std::fmt::Debug for Variables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Variables").finish_non_exhaustive()
    }
}

impl From<Vec<(wit::ArgumentsId, Vec<u8>)>> for Variables {
    fn from(values: Vec<(wit::ArgumentsId, Vec<u8>)>) -> Self {
        Self(values)
    }
}

impl Variables {
    /// Deserializes the arguments of this field into the specified type
    pub fn get<'de, T>(&'de self, id: ArgumentsId) -> Result<T, SdkError>
    where
        T: Deserialize<'de>,
    {
        let bytes = self.get_bytes(id);
        crate::cbor::from_slice(bytes).map_err(Into::into)
    }

    /// Deserializes the arguments of this field into the specified type with the given seed.
    pub fn get_seed<'de, Seed>(&'de self, id: ArgumentsId, seed: Seed) -> Result<Seed::Value, SdkError>
    where
        Seed: DeserializeSeed<'de>,
    {
        let bytes = self.get_bytes(id);
        crate::cbor::from_slice_with_seed(bytes, seed).map_err(Into::into)
    }

    fn get_bytes(&self, id: ArgumentsId) -> &[u8] {
        self.0
            .iter()
            .find_map(|(args_id, args)| if *args_id == id.0 { Some(args.as_slice()) } else { None })
            .unwrap()
    }
}
