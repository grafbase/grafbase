use crate::errors::LoginApiError;
use serde::{Deserialize, Serialize};
pub use server::types::ServerMessage;
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

pub const DATABASE_REGIONS: [DatabaseRegion; 9] = [
    DatabaseRegion::UsEast1,
    DatabaseRegion::UsWest2,
    DatabaseRegion::EuCentral1,
    DatabaseRegion::EuNorth1,
    DatabaseRegion::EuWest1,
    DatabaseRegion::EuWest3,
    DatabaseRegion::ApNortheast2,
    DatabaseRegion::ApSouth1,
    DatabaseRegion::ApSoutheast1,
];

#[derive(Clone)]
pub enum DatabaseRegion {
    ApNortheast2,
    ApSouth1,
    ApSoutheast1,
    EuCentral1,
    EuNorth1,
    EuWest1,
    EuWest3,
    UsEast1,
    UsWest2,
}

impl Display for DatabaseRegion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            Self::EuNorth1 => "eu-north-1",
            Self::ApNortheast2 => "ap-northeast-2",
            Self::ApSouth1 => "ap-south-1",
            Self::ApSoutheast1 => "ap-southeast-1",
            Self::EuCentral1 => "eu-central-1",
            Self::EuWest1 => "eu-west-1",
            Self::EuWest3 => "eu-west-3",
            Self::UsEast1 => "us-east-1",
            Self::UsWest2 => "us-west-2",
        };
        formatter.write_str(display)
    }
}

impl DatabaseRegion {
    #[must_use]
    pub fn to_location_name(&self) -> &'static str {
        match self {
            Self::ApNortheast2 => "Seoul",
            Self::ApSouth1 => "Mumbai",
            Self::ApSoutheast1 => "Singapore",
            Self::EuCentral1 => "Frankfurt",
            Self::EuNorth1 => "Stockholm",
            Self::EuWest1 => "Ireland",
            Self::EuWest3 => "Paris",
            Self::UsEast1 => "N. Virginia",
            Self::UsWest2 => "Oregon",
        }
    }
}
