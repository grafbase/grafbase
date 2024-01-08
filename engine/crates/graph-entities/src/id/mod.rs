use std::{borrow::Cow, fmt::Display};

mod constraint;
pub use constraint::{
    db::{ConstraintID, ConstraintIDError},
    normalize_constraint_value, ConstraintDefinition, ConstraintType,
};

mod node;
pub use node::{NodeID, NodeIDError};

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

impl<'a> TryFrom<&'a str> for ID<'a> {
    type Error = IDError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Ok(constraint) = ConstraintID::try_from(value) {
            return Ok(Self::ConstraintID(constraint));
        }
        if let Ok(node) = NodeID::from_borrowed(value) {
            return Ok(Self::NodeID(node));
        }

        Err(IDError::InvalidID)
    }
}

impl<'a> ID<'a> {
    pub fn is_constraint(&self) -> bool {
        matches!(self, Self::ConstraintID(_))
    }

    pub fn ty(&self) -> Cow<'a, str> {
        match self {
            Self::NodeID(a) => a.ty(),
            Self::ConstraintID(a) => a.ty(),
        }
    }
}
