use std::{borrow::Cow, fmt::Display, str::FromStr};

use serde::Serialize;
use ulid::Ulid;

use super::ID_SEPARATOR;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Hash, Serialize)]
pub struct NodeID<'a> {
    ty: Cow<'a, str>,
    ulid: Cow<'a, str>,
}

impl<'a> AsRef<NodeID<'a>> for NodeID<'a> {
    fn as_ref(&self) -> &NodeID<'a> {
        self
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum NodeIDError {
    #[error("Invalid ID Provided: {0}")]
    InvalidID(String),
}

impl<'a> NodeID<'a> {
    pub fn from_owned(value: String) -> Result<Self, NodeIDError> {
        if let Some((ty, ulid)) = value.split_once(ID_SEPARATOR) {
            if Ulid::from_str(ulid).is_ok() {
                Ok(Self {
                    ty: Cow::Owned(ty.to_lowercase()),
                    ulid: Cow::Owned(ulid.to_string()),
                })
            } else {
                Err(NodeIDError::InvalidID(value))
            }
        } else {
            Err(NodeIDError::InvalidID(value))
        }
    }

    pub fn from_borrowed(value: &'a str) -> Result<Self, NodeIDError> {
        if let Some((ty, ulid)) = value.split_once(ID_SEPARATOR) {
            if Ulid::from_str(ulid).is_ok() {
                Ok(Self {
                    ty: Cow::Owned(ty.to_lowercase()),
                    ulid: Cow::Borrowed(ulid),
                })
            } else {
                Err(NodeIDError::InvalidID(value.to_string()))
            }
        } else {
            Err(NodeIDError::InvalidID(value.to_string()))
        }
    }

    pub fn new(ty: &'a str, ulid: &'a str) -> Self {
        Self {
            ty: Cow::Owned(ty.to_lowercase()),
            ulid: Cow::Borrowed(ulid),
        }
    }

    pub fn new_owned(ty: String, ulid: String) -> Self {
        Self {
            ty: Cow::Owned(ty.to_lowercase()),
            ulid: Cow::Owned(ulid),
        }
    }
}

impl<'a> NodeID<'a> {
    pub fn ty(&self) -> Cow<'a, str> {
        self.ty.clone()
    }

    pub fn ulid(&self) -> Cow<'a, str> {
        self.ulid.clone()
    }
}

impl<'a> Display for NodeID<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{ID_SEPARATOR}{}", self.ty.to_lowercase(), self.ulid)
    }
}

#[cfg(test)]
mod tests {
    use crate::NodeID;

    #[test]
    fn check_ulid_owned() {
        let bad_id = NodeID::from_owned("ty_notulid".to_string());
        let good_id = NodeID::from_owned("ty_01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string());

        assert!(bad_id.is_err());
        assert!(good_id.is_ok());
    }

    #[test]
    fn check_ulid_borrowed() {
        let bad_id = NodeID::from_borrowed("ty_notulid");
        let good_id = NodeID::from_borrowed("ty_01ARZ3NDEKTSV4RRFFQ69G5FAV");

        assert!(bad_id.is_err());
        assert!(good_id.is_ok());
    }
}
