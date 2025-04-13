use operation::{PositionedResponseKey, ResponseKey};
use schema::{ObjectDefinitionId, StringId};

use super::{ResponseInaccessibleValueId, ResponseListId, ResponseMapId, ResponseObjectId};

#[derive(Debug)]
pub(crate) struct ResponseObject {
    pub(super) definition_id: Option<ObjectDefinitionId>,
    /// fields are ordered by the position they appear in the query.
    pub(super) fields_sorted_by_key: Vec<ResponseObjectField>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResponseObjectField {
    pub key: PositionedResponseKey,
    pub value: ResponseValue,
}

impl ResponseObject {
    pub fn new(definition_id: Option<ObjectDefinitionId>, mut fields: Vec<ResponseObjectField>) -> Self {
        fields.sort_unstable_by(|a, b| a.key.cmp(&b.key));
        Self {
            definition_id,
            fields_sorted_by_key: fields,
        }
    }

    pub fn len(&self) -> usize {
        self.fields_sorted_by_key.len()
    }

    pub fn fields(&self) -> impl Iterator<Item = &ResponseObjectField> {
        self.fields_sorted_by_key.iter()
    }

    pub fn find_by_response_key(&self, key: ResponseKey) -> Option<&ResponseValue> {
        self.fields_sorted_by_key
            .iter()
            .find(|field| field.key.response_key == key)
            .map(|field| &field.value)
    }
}

impl std::ops::Index<usize> for ResponseObject {
    type Output = ResponseValue;

    fn index(&self, index: usize) -> &Self::Output {
        &self.fields_sorted_by_key[index].value
    }
}

impl std::ops::IndexMut<usize> for ResponseObject {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.fields_sorted_by_key[index].value
    }
}

/// We keep track of whether a value is nullable or not for error propagation across plans
/// We include directly inside the ResponseValue as it'll be at least have the size of u64 + 1
/// word. As the enum variants don't need the full word, we might as well re-use that extra space
/// for something.
///
/// For the same reason we don't use a boxed slice for `List` to make it easier to for error
/// propagation to change a list item to null. So it's a slice id (offset + length in u32) into a
/// specific ResponseDataPart.
#[derive(Default, Debug, Clone)]
pub(crate) enum ResponseValue {
    #[default]
    Null,
    Boolean {
        value: bool,
    },
    // Defined as i32
    // https://spec.graphql.org/October2021/#sec-Int
    Int {
        value: i32,
    },
    Float {
        value: f64,
    },
    String {
        value: String,
    },
    StringId {
        id: StringId,
    },
    List {
        id: ResponseListId,
    },
    Object {
        id: ResponseObjectId,
    },
    Inaccessible {
        id: ResponseInaccessibleValueId,
    },
    Unexpected,
    // For Any, anything serde_json::Value would support
    I64 {
        value: i64,
    },
    U64 {
        value: u64,
    },
    Map {
        id: ResponseMapId,
    },
}

impl<T: Into<ResponseValue>> From<Option<T>> for ResponseValue {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => value.into(),
            None => Self::Null,
        }
    }
}

impl From<StringId> for ResponseValue {
    fn from(id: StringId) -> Self {
        Self::StringId { id }
    }
}

impl From<bool> for ResponseValue {
    fn from(value: bool) -> Self {
        Self::Boolean { value }
    }
}

impl From<i32> for ResponseValue {
    fn from(value: i32) -> Self {
        Self::Int { value }
    }
}

impl From<i64> for ResponseValue {
    fn from(value: i64) -> Self {
        Self::I64 { value }
    }
}

impl From<f64> for ResponseValue {
    fn from(value: f64) -> Self {
        Self::Float { value }
    }
}

impl From<String> for ResponseValue {
    fn from(value: String) -> Self {
        Self::String { value }
    }
}

impl From<ResponseListId> for ResponseValue {
    fn from(id: ResponseListId) -> Self {
        Self::List { id }
    }
}

impl From<ResponseObjectId> for ResponseValue {
    fn from(id: ResponseObjectId) -> Self {
        Self::Object { id }
    }
}

#[cfg(test)]
#[test]
fn check_response_value_size() {
    assert_eq!(std::mem::size_of::<ResponseValue>(), 24);
    assert_eq!(std::mem::align_of::<ResponseValue>(), 8);
}
