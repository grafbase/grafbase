use futures_util::{stream::FuturesOrdered, FutureExt, StreamExt, TryFutureExt};

use crate::{bindings::component::grafbase::types::Error, error, REQWEST};

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

pub(super) async fn authorize_user(current_user_id: usize, user_ids: Vec<usize>) -> Vec<Result<(), Error>> {
    user_ids
        .into_iter()
        .map(|user_id| async move {
            tracing::info!("Authorizing access to user {} for user {}", user_id, current_user_id);

            REQWEST
                .post("http://localhost:4001/authorize-user")
                .json(&AuthorizeUserRequest {
                    current_user_id,
                    user_id,
                })
                .send()
                .and_then(|response| response.bytes())
                .map(|result| match result {
                    Ok(bytes) => {
                        let response: AuthorizationResponse =
                            serde_json::from_slice(&bytes).expect("Failed to deserialize authorization response");
                        if response.authorized {
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
                .await
        })
        .collect::<FuturesOrdered<_>>()
        .collect()
        .await
}

pub(super) async fn authorize_address(current_user_id: usize, owner_ids: Vec<usize>) -> Vec<Result<(), Error>> {
    owner_ids
        .into_iter()
        .map(|owner_id| async move {
            tracing::info!(
                "Authorizing access to address of user {} for user {}",
                owner_id,
                current_user_id
            );

            REQWEST
                .post("http://localhost:4001/authorize-address")
                .json(&AuthorizeAddressRequest {
                    current_user_id,
                    owner_id,
                })
                .send()
                .and_then(|response| response.bytes())
                .map(|result| match result {
                    Ok(bytes) => {
                        let response: AuthorizationResponse =
                            serde_json::from_slice(&bytes).expect("Failed to deserialize authorization response");
                        if response.authorized {
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
                .await
        })
        .collect::<FuturesOrdered<_>>()
        .collect()
        .await
}
