use std::fmt::{Debug, Display};

use dynaql_parser::Pos;
use thiserror::Error;

use crate::ServerError;

/// The purpose of this structure is to prepare for ID Obfuscation withing Dynaql
pub struct ObfuscatedID<'a> {
    ty: &'a str,
    id: &'a str,
}

const SEPARATOR: char = '_';

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ObfuscatedIDError {
    #[error("You are trying to manipulate an entity with the wrong query.")]
    InvalidType { expected: String, current: String },
    #[error("Something went wrong.")]
    InvalidID,
}
impl ObfuscatedIDError {
    pub fn into_server_error(self, pos: Pos) -> ServerError {
        crate::Error::new_with_source(self).into_server_error(pos)
    }
}

impl<'a> ObfuscatedID<'a> {
    pub fn new(id: &'a str) -> Result<Self, ObfuscatedIDError> {
        match id.rsplit_once(SEPARATOR) {
            Some((ty, id)) => Ok(Self { ty, id }),
            _ => Err(ObfuscatedIDError::InvalidID),
        }
    }

    /// The given ID should be of the expected type.
    pub fn expect(id: &'a str, ty: &'a str) -> Result<Self, ObfuscatedIDError> {
        let id = Self::new(id)?;

        if id.ty.to_lowercase() == ty.to_lowercase() {
            Ok(id)
        } else {
            Err(ObfuscatedIDError::InvalidType {
                expected: ty.to_string(),
                current: id.ty.to_string(),
            })
        }
    }

    pub fn ty(&'a self) -> &'a str {
        self.ty
    }

    pub fn id(&'a self) -> &'a str {
        self.id
    }
}

impl<'a> Display for ObfuscatedID<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{SEPARATOR}{}", self.ty, self.id)
    }
}
