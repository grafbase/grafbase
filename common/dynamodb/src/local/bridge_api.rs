use serde::Deserialize;
use surf::StatusCode;

use super::types::Record;
use super::types::{BridgeUrl, Constraint, Mutation, Operation};

pub async fn query<'a>(operaton: Operation, port: &str) -> Result<Vec<Record>, surf::Error> {
    let response = surf::client()
        .post(BridgeUrl::Query(port).to_string())
        .body_json(&operaton)?
        .await?
        .body_json::<Vec<Record>>()
        .await?;

    Ok(response)
}

#[derive(Deserialize, Debug)]
pub enum ApiErrorKind {
    ConstraintViolation(Constraint),
}

#[allow(dead_code)]
pub enum MutationError {
    Surf(surf::Error),
    InternalServerError,
    Api(ApiError),
}

impl From<surf::Error> for MutationError {
    fn from(error: surf::Error) -> Self {
        Self::Surf(error)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    #[serde(flatten)]
    pub error_kind: ApiErrorKind,
}

pub async fn mutation<'a>(mutations: Vec<Operation>, port: &str) -> Result<(), MutationError> {
    let client = surf::client();
    let mut response = client
        .post(BridgeUrl::Mutation(port).to_string())
        .body_json(&Mutation { mutations })?
        .await?;

    match response.status() {
        StatusCode::Conflict => {
            let error = response.body_json::<ApiError>().await?;
            return Err(MutationError::Api(error));
        }
        StatusCode::InternalServerError => {
            return Err(MutationError::InternalServerError);
        }
        _ => {}
    }

    Ok(())
}
