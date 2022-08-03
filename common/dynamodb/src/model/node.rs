use super::id::ID_SEPARATOR;
use dynomite::{Attribute, AttributeError};
use std::borrow::Cow;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NodeID<'a> {
    ty: Cow<'a, str>,
    ulid: Cow<'a, str>,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum IDError {
    #[error("Invalid ID Provided: {0}")]
    InvalidID(String),
}

impl<'a> NodeID<'a> {
    pub fn from_owned(value: String) -> Result<Self, IDError> {
        if let Some((ty, ulid)) = value.split_once(ID_SEPARATOR) {
            Ok(Self {
                ty: Cow::Owned(ty.to_lowercase()),
                ulid: Cow::Owned(ulid.to_string()),
            })
        } else {
            Err(IDError::InvalidID(value))
        }
    }

    pub fn from_borrowed(value: &'a str) -> Result<Self, IDError> {
        if let Some((ty, ulid)) = value.split_once(ID_SEPARATOR) {
            Ok(Self {
                ty: Cow::Owned(ty.to_lowercase()),
                ulid: Cow::Borrowed(ulid),
            })
        } else {
            Err(IDError::InvalidID(value.to_string()))
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

impl<'a> Attribute for NodeID<'a> {
    fn into_attr(self) -> dynomite::AttributeValue {
        self.to_string().into_attr()
    }

    fn from_attr(value: dynomite::AttributeValue) -> Result<Self, dynomite::AttributeError> {
        Self::from_owned(value.s.ok_or(AttributeError::InvalidType)?).map_err(|_| AttributeError::InvalidFormat)
    }
}
