use std::collections::BTreeMap;

use schema::StringId;

use super::{ResponseEdge, ResponseKey, ResponseListId, ResponseObjectId};

#[derive(Debug)]
pub struct ResponseObject {
    // fields are ordered by the position they appear in the query.
    pub fields: BTreeMap<ResponseEdge, ResponseValue>,
}

impl ResponseObject {
    // Until acutal field collection with the concrete object id we're not certain of which bound
    // response key (field position & name) will be used but the actual response key (field name)
    // should still be there. So, first trying with the bound key and then searching for a matching
    // response key. This is only used for executor input creation, usually a few fields, and may
    // only fallback if the selection had type conditions and field duplication.
    // So should be a decent tradeoff as this allows us to serialize the whole response without any
    // additional metadata as both position and key are encoded.
    pub(super) fn find(&self, edge: ResponseEdge) -> Option<&ResponseValue> {
        self.fields
            .get(&edge)
            .or_else(|| edge.as_response_key().and_then(|key| self.find_by_name(key)))
    }

    fn find_by_name(&self, target: ResponseKey) -> Option<&ResponseValue> {
        self.fields.iter().find_map(|(key, field)| match key.unpack() {
            super::UnpackedResponseEdge::BoundResponseKey(key) if ResponseKey::from(key) == target => Some(field),
            _ => None,
        })
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
pub enum ResponseValue {
    #[default]
    Null,
    Boolean {
        value: bool,
        nullable: bool,
    },
    // Defined as i32
    // https://spec.graphql.org/October2021/#sec-Int
    Int {
        value: i32,
        nullable: bool,
    },
    BigInt {
        value: i64,
        nullable: bool,
    },
    Float {
        value: f64,
        nullable: bool,
    },
    String {
        value: Box<str>,
        nullable: bool,
    },
    StringId {
        id: StringId,
        nullable: bool,
    },
    Json {
        value: Box<serde_json::Value>,
        nullable: bool,
    },
    List {
        id: ResponseListId,
        nullable: bool,
    },
    Object {
        id: ResponseObjectId,
        nullable: bool,
    },
}

impl ResponseValue {
    pub(super) fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub(super) fn into_nullable(self) -> Self {
        match self {
            Self::Null => Self::Null,
            Self::Boolean { value, .. } => Self::Boolean { value, nullable: true },
            Self::Int { value, .. } => Self::Int { value, nullable: true },
            Self::BigInt { value, .. } => Self::BigInt { value, nullable: true },
            Self::Float { value, .. } => Self::Float { value, nullable: true },
            Self::String { value, .. } => Self::String { value, nullable: true },
            Self::StringId { id, .. } => Self::StringId { id, nullable: true },
            Self::Json { value, .. } => Self::Json { value, nullable: true },
            Self::List { id, .. } => Self::List { id, nullable: true },
            Self::Object { id, .. } => Self::Object { id, nullable: true },
        }
    }
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
        Self::StringId { id, nullable: false }
    }
}

impl From<bool> for ResponseValue {
    fn from(value: bool) -> Self {
        Self::Boolean { value, nullable: false }
    }
}

impl From<ResponseListId> for ResponseValue {
    fn from(id: ResponseListId) -> Self {
        Self::List { id, nullable: false }
    }
}

impl From<ResponseObjectId> for ResponseValue {
    fn from(id: ResponseObjectId) -> Self {
        Self::Object { id, nullable: false }
    }
}
