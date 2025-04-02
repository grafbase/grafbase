use std::borrow::Cow;

use crate::{SdkError, wit::selection_set_resolver_types as wit};
use serde::{Deserialize, de::DeserializeSeed};

/// A field within a GraphQL query
#[derive(Clone, Copy)]
pub struct Field<'a> {
    pub(crate) fields: &'a [wit::Field],
    pub(crate) field: &'a wit::Field,
}

impl<'a> Field<'a> {
    /// Gets the alias of this field, if any
    pub fn alias(&self) -> Option<&str> {
        self.field.alias.as_deref()
    }

    /// Gets the arguments ID of this field, if any
    pub fn arguments_id(&self) -> Option<ArgumentsId> {
        self.field.arguments.map(ArgumentsId)
    }

    /// Deserializes the arguments of this field into the specified type
    pub fn arguments<'de, T>(&self, values: ArgumentValues<'de>) -> Result<T, SdkError>
    where
        T: Deserialize<'de>,
    {
        match self.field.arguments {
            Some(id) => values.get(id.into()),
            None => Err(SdkError::from("Field has no arguments".to_string())),
        }
    }

    /// Deserializes the arguments of this field into the specified type with the given seed.
    pub fn arguments_seed<'de, Seed>(&self, seed: Seed, values: ArgumentValues<'de>) -> Result<Seed::Value, SdkError>
    where
        Seed: DeserializeSeed<'de>,
    {
        match self.field.arguments {
            Some(id) => values.get_seed(id.into(), seed),
            None => Err(SdkError::from("Field has no arguments".to_string())),
        }
    }

    /// Gets the selection set of this field, if any
    pub fn selection_set(&self) -> Option<SelectionSet<'a>> {
        self.field.selection_set.map(|selection_set| SelectionSet {
            fields: self.fields,
            selection_set,
        })
    }

    /// Serialize the field and its selection set
    pub fn into_bytes(&self) -> Vec<u8> {
        postcard::to_stdvec(&Data {
            fields: Cow::Borrowed(self.fields),
            ix: element_offset(self.fields, self.field).unwrap(),
        })
        .unwrap()
    }

    /// Deserialize a field and its selection set
    pub fn with_bytes<T>(data: &[u8], f: impl FnOnce(Field<'_>) -> T) -> Result<T, SdkError> {
        match postcard::from_bytes(data) {
            Ok(Data { fields, ix }) => {
                let field = fields.get(ix).ok_or("Field index out of bounds")?;
                Ok(f(Field {
                    fields: fields.as_ref(),
                    field,
                }))
            }
            Err(err) => Err(format!("Failed to deserialize field data: {err}").into()),
        }
    }
}

// std::lice::element_offset which is unstable
fn element_offset(slice: &[wit::Field], element: &wit::Field) -> Option<usize> {
    let self_start = slice.as_ptr().addr();
    let elem_start = std::ptr::from_ref(element).addr();

    let byte_offset = elem_start.wrapping_sub(self_start);

    if byte_offset % std::mem::size_of::<wit::Field>() != 0 {
        return None;
    }

    let offset = byte_offset / std::mem::size_of::<wit::Field>();

    if offset < slice.len() { Some(offset) } else { None }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Data<'a> {
    fields: Cow<'a, [wit::Field]>,
    ix: usize,
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
    /// Iterator over the fields in this selection set
    pub fn fields(&self) -> impl ExactSizeIterator<Item = Field<'a>> + '_ {
        let (start, end) = self.selection_set.fields_ordered_by_parent_entity;
        let fields = self.fields;
        fields[usize::from(start)..usize::from(end)]
            .iter()
            .map(move |field| Field { fields, field })
    }

    /// Whether this selection set requires a `__typename` field
    pub fn requires_typename(&self) -> bool {
        self.selection_set.requires_typename
    }
}

/// Identifier for arguments in a GraphQL query
#[derive(Clone, Copy)]
pub struct ArgumentsId(wit::ArgumentsId);

impl From<wit::ArgumentsId> for ArgumentsId {
    fn from(id: wit::ArgumentsId) -> Self {
        Self(id)
    }
}

/// All argument values for a given selection set, to be used with [Field].
#[derive(Clone, Copy)]
pub struct ArgumentValues<'a>(pub(crate) &'a [(wit::ArgumentsId, Vec<u8>)]);

impl<'a> ArgumentValues<'a> {
    /// Deserializes the arguments of this field into the specified type
    pub fn get<T>(&self, id: ArgumentsId) -> Result<T, SdkError>
    where
        T: Deserialize<'a>,
    {
        let bytes = self.get_bytes(id);
        crate::cbor::from_slice(bytes).map_err(Into::into)
    }

    /// Deserializes the arguments of this field into the specified type with the given seed.
    pub fn get_seed<Seed>(&self, id: ArgumentsId, seed: Seed) -> Result<Seed::Value, SdkError>
    where
        Seed: DeserializeSeed<'a>,
    {
        let bytes = self.get_bytes(id);
        crate::cbor::from_slice_with_seed(bytes, seed).map_err(Into::into)
    }

    fn get_bytes(&self, id: ArgumentsId) -> &'a [u8] {
        self.0
            .iter()
            .find_map(|(args_id, args)| if *args_id == id.0 { Some(args.as_slice()) } else { None })
            .unwrap()
    }
}
