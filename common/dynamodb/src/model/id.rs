use dynomite::{Attribute, AttributeError};
use std::fmt::Display;

use super::constraint::db::ConstraintID;
use super::node::NodeID;

pub const ID_SEPARATOR: char = '_';

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ID<'a> {
    NodeID(NodeID<'a>),
    ConstraintID(ConstraintID<'a>),
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum IDError {
    #[error("Invalid ID Provided")]
    InvalidID,
}

impl<'a> Display for ID<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NodeID(a) => a.fmt(f),
            Self::ConstraintID(a) => a.fmt(f),
        }
    }
}

impl<'a> TryFrom<String> for ID<'a> {
    type Error = IDError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Ok(constraint) = ConstraintID::try_from(value.clone()) {
            return Ok(Self::ConstraintID(constraint));
        }
        if let Ok(node) = NodeID::from_owned(value) {
            return Ok(Self::NodeID(node));
        }

        Err(IDError::InvalidID)
    }
}

impl<'a> Attribute for ID<'a> {
    fn into_attr(self) -> dynomite::AttributeValue {
        self.to_string().into_attr()
    }

    fn from_attr(value: dynomite::AttributeValue) -> Result<Self, dynomite::AttributeError> {
        Self::try_from(value.s.ok_or(AttributeError::InvalidType)?).map_err(|_| AttributeError::InvalidFormat)
    }
}
