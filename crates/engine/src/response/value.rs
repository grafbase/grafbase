use operation::{PositionedResponseKey, ResponseKey};
use schema::{ObjectDefinitionId, StringId};

use crate::response::{PartOwnedFieldsId, PartSharedFieldsId};

use super::{ResponseInaccessibleValueId, ResponseListId, ResponseMapId, ResponseObjectId};

#[derive(Debug)]
pub(crate) struct ResponseObject {
    pub(super) definition_id: Option<ObjectDefinitionId>,
    pub(super) fields_sorted_by_key: ResponseFieldsSortedByKey,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ResponseFieldsSortedByKey {
    Slice {
        fields_id: PartSharedFieldsId,
        offset: u32,
        // More than u16::MAX fields would have broken at the operation preparation already
        limit: u16,
    },
    Owned {
        fields_id: PartOwnedFieldsId,
    },
}

impl From<PartOwnedFieldsId> for ResponseFieldsSortedByKey {
    fn from(id: PartOwnedFieldsId) -> Self {
        Self::Owned { fields_id: id }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ResponseField {
    pub key: PositionedResponseKey,
    pub value: ResponseValue,
}

impl ResponseObject {
    pub fn new(definition_id: Option<ObjectDefinitionId>, fields: impl Into<ResponseFieldsSortedByKey>) -> Self {
        Self {
            definition_id,
            fields_sorted_by_key: fields.into(),
        }
    }
}

impl ResponseField {
    pub fn null() -> Self {
        Self {
            key: PositionedResponseKey {
                // SAFETY: We're calling it after an operation has been properly parsed. We
                // wouldn't be generated a response otherwise.
                response_key: unsafe { ResponseKey::null() },
                query_position: None,
            },
            value: ResponseValue::Null,
        }
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
        offset: u32,
        limit: u32,
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
        offset: u32,
        limit: u32,
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

impl From<ResponseObjectId> for ResponseValue {
    fn from(id: ResponseObjectId) -> Self {
        Self::Object { id }
    }
}

impl From<ResponseInaccessibleValueId> for ResponseValue {
    fn from(id: ResponseInaccessibleValueId) -> Self {
        Self::Inaccessible { id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::mem::size_of;

    #[test]
    fn size_of_response_value() {
        assert_eq!(size_of::<ResponseValue>(), 24);
    }
    #[test]
    fn size_of_response_fields() {
        assert_eq!(size_of::<ResponseFieldsSortedByKey>(), 12);
    }
    #[test]
    fn size_of_response_field() {
        assert_eq!(size_of::<ResponseField>(), 32);
    }
    #[test]
    fn size_of_response_object() {
        assert_eq!(size_of::<ResponseObject>(), 16);
    }
}
