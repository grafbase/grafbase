use reqwest::StatusCode;
use send_wrapper::SendWrapper;
use serde::Deserialize;

use std::future::Future;
use std::pin::Pin;

use super::types::Record;
use super::types::{BridgeUrl, Constraint, Mutation, Operation};

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

pub fn query<'a>(
    operaton: Operation,
    port: &str,
) -> Pin<Box<dyn Future<Output = Result<Vec<Record>, QueryError>> + Send + '_>> {
    let request = reqwest::Client::new()
        .post(BridgeUrl::Query(port).to_string())
        .json(&operaton);
    Box::pin(SendWrapper::new(async move {
        let response = request.send().await?;

        if response.status() == StatusCode::INTERNAL_SERVER_ERROR {
            Err(QueryError::InternalServerError)
        } else {
            Ok(response.json::<Vec<Record>>().await?)
        }
    }))
}

pub fn mutation<'a>(
    mutations: Vec<Operation>,
    port: &str,
) -> Pin<Box<dyn Future<Output = Result<(), MutationError>> + Send + '_>> {
    let client = reqwest::Client::new();
    let request = client
        .post(BridgeUrl::Mutation(port).to_string())
        .json(&Mutation { mutations });
    Box::pin(SendWrapper::new(async move {
        let response = request.send().await?;
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
    }))
}
