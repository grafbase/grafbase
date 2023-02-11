use serde::Deserialize;
use surf::StatusCode;

use super::types::Record;
use super::types::{BridgeUrl, Constraint, Mutation, Operation, ResolverInvocation};

pub async fn query<'a>(operaton: Operation, port: &str) -> Result<Vec<Record>, QueryError> {
    let mut response = surf::client()
        .post(BridgeUrl::Query(port).to_string())
        .body_json(&operaton)?
        .await?;

    if response.status() == StatusCode::InternalServerError {
        Err(QueryError::InternalServerError)
    } else {
        Ok(response.body_json::<Vec<Record>>().await?)
    }
}

#[derive(Deserialize, Debug)]
pub enum ApiErrorKind {
    ConstraintViolation(Constraint),
}

#[allow(dead_code)]
pub enum QueryError {
    Surf(surf::Error),
    InternalServerError,
}

impl From<surf::Error> for QueryError {
    fn from(error: surf::Error) -> Self {
        Self::Surf(error)
    }
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

#[allow(dead_code)]
pub enum ResolverInvocationError {
    Surf(surf::Error),
    InternalServerError,
}

impl From<surf::Error> for ResolverInvocationError {
    fn from(error: surf::Error) -> Self {
        Self::Surf(error)
    }
}

pub async fn invoke_resolver<'a>(
    resolver_name: &str,
    arguments: serde_json::Value,
    port: &str,
) -> Result<serde_json::Value, ResolverInvocationError> {
    let client = surf::client();
    let mut response = client
        .post(BridgeUrl::InvokeResolver(port).to_string())
        .body_json(&ResolverInvocation {
            resolver_name,
            arguments,
        })?
        .await?;

    match response.status() {
        StatusCode::InternalServerError => {
            return Err(ResolverInvocationError::InternalServerError);
        }
        _ => {}
    }

    Ok(response.body_json::<serde_json::Value>().await?)
}
