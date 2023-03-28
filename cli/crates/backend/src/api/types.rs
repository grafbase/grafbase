use super::{errors::LoginApiError, graphql};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

pub enum LoginMessage {
    CallbackUrl(String),
    Done,
    Error(LoginApiError),
}

#[derive(Debug)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub personal: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credentials<'a> {
    pub access_token: &'a str,
}

impl<'a> ToString for Credentials<'a> {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).expect("must parse")
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    pub account_id: String,
    pub project_id: String,
}

impl ToString for ProjectMetadata {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).expect("must parse")
    }
}

#[derive(Clone)]
pub struct DatabaseRegion {
    pub name: String,
    pub city: String,
}

impl Display for DatabaseRegion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.name)
    }
}

impl From<graphql::queries::DatabaseRegion> for DatabaseRegion {
    fn from(api_region: graphql::queries::DatabaseRegion) -> Self {
        Self {
            name: api_region.name,
            city: api_region.city,
        }
    }
}
