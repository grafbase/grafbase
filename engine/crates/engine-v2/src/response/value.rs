use schema::StringId;

use super::ResponseObjectId;
use crate::execution::StrId;

#[derive(Default, Debug)]
pub enum ResponseValue {
    #[default]
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    StringId(StringId),
    StrId(StrId),
    List(Vec<ResponseValue>),
    Object(ResponseObjectId),
}

impl FromIterator<ResponseValue> for ResponseValue {
    fn from_iter<T: IntoIterator<Item = ResponseValue>>(iter: T) -> Self {
        ResponseValue::List(iter.into_iter().collect())
    }
}

impl ResponseValue {
    pub fn as_object(&self) -> Option<ResponseObjectId> {
        match self {
            Self::Object(id) => Some(*id),
            _ => None,
        }
    }
}

impl<T> From<Option<T>> for ResponseValue
where
    ResponseValue: From<T>,
{
    fn from(value: Option<T>) -> Self {
        value.map_or(ResponseValue::Null, ResponseValue::from)
    }
}

impl From<StrId> for ResponseValue {
    fn from(value: StrId) -> Self {
        ResponseValue::StrId(value)
    }
}

impl From<StringId> for ResponseValue {
    fn from(value: StringId) -> Self {
        ResponseValue::StringId(value)
    }
}

impl From<String> for ResponseValue {
    fn from(value: String) -> Self {
        ResponseValue::String(value)
    }
}

impl From<&str> for ResponseValue {
    fn from(value: &str) -> Self {
        ResponseValue::String(value.to_string())
    }
}

impl From<bool> for ResponseValue {
    fn from(value: bool) -> Self {
        ResponseValue::Bool(value)
    }
}
