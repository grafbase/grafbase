use std::borrow::Cow;

use schema::DirectiveSiteId;

use crate::{Location, OperationAttributes};

pub type Result<T> = std::result::Result<T, Errors>;

#[derive(Debug)]
pub struct Errors {
    pub items: Vec<Error>,
    pub attributes: Option<OperationAttributes>,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    Parsing,
    Validation,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: Cow<'static, str>,
    pub locations: Vec<Location>,
    pub site_id: Option<DirectiveSiteId>,
}

impl Error {
    pub(crate) fn parsing(message: impl Into<Cow<'static, str>>) -> Self {
        Error {
            kind: ErrorKind::Parsing,
            message: message.into(),
            locations: Vec::new(),
            site_id: None,
        }
    }

    pub(crate) fn validation(message: impl Into<Cow<'static, str>>) -> Self {
        Error {
            kind: ErrorKind::Validation,
            message: message.into(),
            locations: Vec::new(),
            site_id: None,
        }
    }

    pub(crate) fn with_maybe_site_id(mut self, site_id: Option<DirectiveSiteId>) -> Self {
        self.site_id = site_id;
        self
    }

    pub(crate) fn with_location(mut self, location: Location) -> Self {
        self.locations.push(location);
        self
    }

    pub(crate) fn with_locations(mut self, locations: impl IntoIterator<Item = Location>) -> Self {
        self.locations.extend(locations);
        self
    }
}
