use std::fmt;

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum OperationType {
    FindOne,
    FindMany,
    InsertOne,
    DeleteOne,
}

impl AsRef<str> for OperationType {
    fn as_ref(&self) -> &str {
        match self {
            Self::FindOne => "findOne",
            Self::FindMany => "find",
            Self::InsertOne => "insertOne",
            Self::DeleteOne => "deleteOne",
        }
    }
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}
