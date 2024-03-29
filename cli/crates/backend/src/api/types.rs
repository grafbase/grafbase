use super::errors::LoginApiError;
use serde::{Deserialize, Serialize};

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
    pub project_id: String,
}

impl ToString for ProjectMetadata {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).expect("must parse")
    }
}
