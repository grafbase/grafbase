use super::types::Record;
use super::types::{BridgePayload, BridgeUrl};

pub async fn query(query: &str, variables: &Vec<String>, port: &str) -> Result<Vec<Record>, surf::Error> {
    let response = surf::client()
        .post(BridgeUrl::Query(port).to_string())
        .body_json(&BridgePayload { query, variables })?
        .await?
        .body_json::<Vec<Record>>()
        .await?;

    Ok(response)
}

pub async fn mutation(query: &str, variables: &Vec<String>, port: &str) -> Result<(), surf::Error> {
    surf::client()
        .post(BridgeUrl::Mutation(port).to_string())
        .body_json(&BridgePayload { query, variables })?
        .await?;

    Ok(())
}
