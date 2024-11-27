use grafbase_hooks::{
    http_client::{self, HttpMethod, HttpRequest},
    Error,
};
use itertools::Itertools;

use crate::error;

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthorizeUserRequest {
    current_user_id: usize,
    user_id: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthorizeAddressRequest {
    current_user_id: usize,
    owner_id: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthorizationResponse {
    authorized: bool,
}

pub(super) fn authorize_user(current_user_id: usize, user_ids: Vec<usize>) -> Vec<Result<(), Error>> {
    let requests = user_ids
        .into_iter()
        .map(|user_id| {
            tracing::info!(
                "Authorizing access to user of user {} for user {}",
                user_id,
                current_user_id
            );

            HttpRequest {
                method: HttpMethod::Post,
                url: String::from("http://localhost:4001/authorize-user"),
                headers: vec![("Content-Type".to_string(), "application/json".to_string())],
                body: serde_json::to_vec(&AuthorizeUserRequest {
                    current_user_id,
                    user_id,
                })
                .unwrap(),
                timeout_ms: Some(1000),
            }
        })
        .collect_vec();

    http_client::execute_many(&requests)
        .into_iter()
        .map(|result| match result {
            Ok(response) => {
                let body: AuthorizationResponse =
                    serde_json::from_slice(&response.body).expect("Failed to deserialize authorization response");

                if body.authorized {
                    Ok(())
                } else {
                    Err(error("Unauthorized"))
                }
            }
            Err(err) => {
                tracing::error!("Auth service request failure: {err:?}");
                Err(error("Unauthorized"))
            }
        })
        .collect()
}

pub(super) fn authorize_address(current_user_id: usize, owner_ids: Vec<usize>) -> Vec<Result<(), Error>> {
    let requests = owner_ids
        .into_iter()
        .map(|owner_id| {
            tracing::info!(
                "Authorizing access to address of user {} for user {}",
                owner_id,
                current_user_id
            );

            HttpRequest {
                method: HttpMethod::Post,
                url: String::from("http://localhost:4001/authorize-user"),
                headers: vec![("Content-Type".to_string(), "application/json".to_string())],
                body: serde_json::to_vec(&AuthorizeAddressRequest {
                    current_user_id,
                    owner_id,
                })
                .unwrap(),
                timeout_ms: Some(1000),
            }
        })
        .collect_vec();

    http_client::execute_many(&requests)
        .into_iter()
        .map(|result| match result {
            Ok(response) => {
                let body: AuthorizationResponse =
                    serde_json::from_slice(&response.body).expect("Failed to deserialize authorization response");

                if body.authorized {
                    Ok(())
                } else {
                    Err(error("Unauthorized"))
                }
            }
            Err(err) => {
                tracing::error!("Auth service request failure: {err:?}");
                Err(error("Unauthorized"))
            }
        })
        .collect()
}
