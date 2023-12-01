use std::collections::BTreeMap;

use schema::{ObjectId, StringId};

use super::{BoundResponseKey, ResponseKey, ResponseListId, ResponseObjectId};

#[derive(Debug)]
pub struct ResponseObject {
    pub object_id: ObjectId,
    // fields are ordered by the position they appear in the query.
    pub fields: BTreeMap<BoundResponseKey, ResponseValue>,
}

impl ResponseObject {
    // Until acutal field collection with the concre object id we're not certain of which bound
    // response key (field position & name) will be used but the actual response key (field name)
    // should still be there. So trying first with the actual bound key as this will be the most
    // common case and iterating over the fields otherwise. This is only used for executor input creation
    // so usually a few fields and will only fallback if the selection had type conditions and field duplication.
    // So should be a decent tradeoff as this allows us to serialize the whole response without any
    // additional metadata as both position and key as encoded.
    pub(super) fn find(&self, key: BoundResponseKey) -> Option<&ResponseValue> {
        self.fields.get(&key).or_else(|| self.find_by_name(key.into()))
    }

    pub(super) fn find_by_name(&self, target: ResponseKey) -> Option<&ResponseValue> {
        for (key, field) in &self.fields {
            if ResponseKey::from(*key) == target {
                return Some(field);
            }
        }
        None
    }
}

/// We keep track of whether a value is nullable or not for error propagation across plans
/// We include directly inside the ResponseValue as it'll be at least have the size of u64 + 1
/// word. As the enum variants don't need the full word, we might as well re-use that extra space
/// for something.
///
/// For the same reason we don't use a boxed slice for `List` to make it easier to for error
/// propagation to change a list item to null. So it's a slice id (offset + length in u32) into a
/// specific ResponseDataPart. As we're doing for `List`, I also did for `String` as this allows
/// ResponseValue to be only `u64 + enum overhead + word padding` long instead of
/// `max(2 words, u64) + enum overhead + word padding`.
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
        // Maybe we should the same a lists, might be more memory efficient
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
