use std::borrow::Cow;

use crate::{Location, OperationAttributes};

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{message}")]
    Parsing {
        message: Cow<'static, str>,
        locations: Vec<Location>,
    },
    #[error("{message}")]
    Validation {
        message: Cow<'static, str>,
        locations: Vec<Location>,
        attributes: OperationAttributes,
    },
}

impl Error {
    pub(crate) fn parsing(message: impl Into<Cow<'static, str>>) -> Self {
        Error::Parsing {
            message: message.into(),
            locations: Vec::new(),
        }
    }

    pub(crate) fn validation(message: impl Into<Cow<'static, str>>, attributes: OperationAttributes) -> Self {
        Error::Validation {
            message: message.into(),
            locations: Vec::new(),
            attributes,
        }
    }

    pub(crate) fn with_location(mut self, location: Location) -> Self {
        match &mut self {
            Error::Parsing { locations, .. } | Error::Validation { locations, .. } => locations.push(location),
        }
        self
    }

    pub(crate) fn with_locations(mut self, locations: impl IntoIterator<Item = Location>) -> Self {
        match &mut self {
            Error::Parsing { locations: loc, .. } | Error::Validation { locations: loc, .. } => loc.extend(locations),
        }
        self
    }
}
