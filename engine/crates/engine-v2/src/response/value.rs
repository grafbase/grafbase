use schema::{RequiredFieldId, StringId};

use super::{ResponseDataPartId, ResponseEdge, ResponseListId, ResponseObjectId};

// Threshold defined a bit arbitrarily
pub const NULL: ResponseValue = ResponseValue::Null;

#[derive(Default, Debug)]
pub(crate) struct ResponseObject {
    /// fields are ordered by the position they appear in the query.
    /// We use ResponseEdge here, but it'll never be an index out of the 3 possible variants.
    /// That's something we should rework at some point, but it's convenient for now.
    fields: Vec<ResponseObjectField>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResponseObjectField {
    pub edge: ResponseEdge,
    pub required_field_id: Option<RequiredFieldId>,
    pub value: ResponseValue,
}

impl ResponseObject {
    pub fn new(mut fields: Vec<ResponseObjectField>) -> Self {
        fields.sort_unstable_by(|a, b| a.edge.cmp(&b.edge));
        Self { fields }
    }

    pub fn extend(&mut self, fields: Vec<ResponseObjectField>) {
        self.fields.extend(fields);
        self.fields.sort_unstable_by(|a, b| a.edge.cmp(&b.edge));
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn fields(&self) -> impl Iterator<Item = &ResponseObjectField> {
        self.fields.iter()
    }

    // FIXME: Shouldn't store by edge nor should the response path...
    pub(super) fn field_position(&self, edge: ResponseEdge) -> Option<usize> {
        self.fields
            .binary_search_by(|field| field.edge.cmp(&edge))
            .ok()
            .or_else(|| match edge.as_response_key() {
                Some(key) => {
                    return self
                        .fields
                        .iter()
                        .position(|field| field.edge.as_response_key() == Some(key));
                }
                None => None,
            })
    }

    pub(super) fn find_required_field(&self, id: RequiredFieldId) -> Option<&ResponseValue> {
        self.fields
            .iter()
            .find(|field| field.required_field_id == Some(id))
            .map(|field| &field.value)
    }
}

impl std::ops::Index<usize> for ResponseObject {
    type Output = ResponseValue;

    fn index(&self, index: usize) -> &Self::Output {
        &self.fields[index].value
    }
}

impl std::ops::IndexMut<usize> for ResponseObject {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.fields[index].value
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
    // Ideally we would use ResponseListId and ResponseObjectId, but those are already padded by
    // Rust. So we miss the opportunity to include the nullable flag and the enum tag in that
    // padding. And we really want ResponseValue to be as small as possible.
    List {
        part_id: ResponseDataPartId,
        offset: u32,
        length: u32,
        nullable: bool,
    },
    Object {
        part_id: ResponseDataPartId,
        index: u32,
        nullable: bool,
    },
}

impl ResponseValue {
    pub(super) fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub(super) fn into_nullable(mut self) -> Self {
        match &mut self {
            Self::Null => (),
            Self::Boolean { nullable, .. } => *nullable = true,
            Self::Int { nullable, .. } => *nullable = true,
            Self::BigInt { nullable, .. } => *nullable = true,
            Self::Float { nullable, .. } => *nullable = true,
            Self::String { nullable, .. } => *nullable = true,
            Self::StringId { nullable, .. } => *nullable = true,
            Self::Json { nullable, .. } => *nullable = true,
            Self::List { nullable, .. } => *nullable = true,
            Self::Object { nullable, .. } => *nullable = true,
        };
        self
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

impl From<i32> for ResponseValue {
    fn from(value: i32) -> Self {
        Self::Int { value, nullable: false }
    }
}

impl From<i64> for ResponseValue {
    fn from(value: i64) -> Self {
        Self::BigInt { value, nullable: false }
    }
}

impl From<f64> for ResponseValue {
    fn from(value: f64) -> Self {
        Self::Float { value, nullable: false }
    }
}

impl From<String> for ResponseValue {
    fn from(value: String) -> Self {
        Self::String {
            value: value.into_boxed_str(),
            nullable: false,
        }
    }
}

impl From<Box<serde_json::Value>> for ResponseValue {
    fn from(value: Box<serde_json::Value>) -> Self {
        Self::Json { value, nullable: false }
    }
}

impl From<ResponseListId> for ResponseValue {
    fn from(id: ResponseListId) -> Self {
        Self::List {
            part_id: id.part_id,
            offset: id.offset,
            length: id.length,
            nullable: false,
        }
    }
}

impl From<ResponseObjectId> for ResponseValue {
    fn from(id: ResponseObjectId) -> Self {
        Self::Object {
            part_id: id.part_id,
            index: id.index,
            nullable: false,
        }
    }
}
