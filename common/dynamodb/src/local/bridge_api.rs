use reqwest::StatusCode;
use serde::Deserialize;

use super::types::Record;
use super::types::{BridgeUrl, Constraint, Mutation, Operation};

pub async fn query<'a>(operaton: Operation, port: &str) -> Result<Vec<Record>, QueryError> {
    let response = reqwest::Client::new()
        .post(BridgeUrl::Query(port).to_string())
        .json(&operaton)
        .send()
        .await?;

    if response.status() == StatusCode::INTERNAL_SERVER_ERROR {
        Err(QueryError::InternalServerError)
    } else {
        Ok(response.json::<Vec<Record>>().await?)
    }
}

#[derive(Deserialize, Debug)]
pub enum ApiErrorKind {
    ConstraintViolation(Constraint),
}

#[allow(dead_code)]
pub enum QueryError {
    Reqwest(reqwest::Error),
    InternalServerError,
}

impl From<reqwest::Error> for QueryError {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(error)
    }
}

#[allow(dead_code)]
pub enum MutationError {
    Reqwest(reqwest::Error),
    InternalServerError,
    Api(ApiError),
}

impl From<reqwest::Error> for MutationError {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(error)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    #[serde(flatten)]
    pub error_kind: ApiErrorKind,
}

pub async fn mutation<'a>(mutations: Vec<Operation>, port: &str) -> Result<(), MutationError> {
    let client = reqwest::Client::new();
    let response = client
        .post(BridgeUrl::Mutation(port).to_string())
        .json(&Mutation { mutations })
        .send()
        .await?;

    match response.status() {
        StatusCode::CONFLICT => {
            let error = response.json::<ApiError>().await?;
            return Err(MutationError::Api(error));
        }
        StatusCode::INTERNAL_SERVER_ERROR => {
            return Err(MutationError::InternalServerError);
        }
        _ => {}
    }

    Ok(())
}
