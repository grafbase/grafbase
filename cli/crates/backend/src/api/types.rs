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

#[derive(Debug)]
pub struct AccountWithProjects {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub personal: bool,
    pub projects: Vec<Project>,
}

#[derive(Debug)]
pub struct Project {
    pub id: String,
    pub slug: String,
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

impl From<graphql::queries::viewer_and_regions::DatabaseRegion> for DatabaseRegion {
    fn from(api_region: graphql::queries::viewer_and_regions::DatabaseRegion) -> Self {
        Self {
            name: api_region.name,
            city: api_region.city,
        }
    }
}
