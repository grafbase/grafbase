use std::{fmt::Display, ops::Deref};

use chrono::{DateTime, SecondsFormat, Utc};
use dynomite::{Attribute, AttributeValue};

// TODO: It should be placed in a different crate than dynamodb
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CurrentDateTime(DateTime<Utc>);

impl CurrentDateTime {
    pub fn new() -> Self {
        Self(Utc::now())
    }

    // Overriding Dynamite implementation as we only store for milliseconds.
    // We probably don't need it, but it's consistent with the previous behavior.
    pub fn into_attr(self) -> AttributeValue {
        self.to_string().into_attr()
    }
}

impl Display for CurrentDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_rfc3339_opts(SecondsFormat::Millis, true))
    }
}

impl From<CurrentDateTime> for DateTime<Utc> {
    fn from(item: CurrentDateTime) -> Self {
        item.0
    }
}

// Suggested by Clippy
impl Default for CurrentDateTime {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for CurrentDateTime {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
