use super::types::Credentials;
use super::{consts::CREDENTIALS_FILE, errors::ApiError};
use crate::consts::USER_AGENT;
use axum::http::{HeaderMap, HeaderValue};
use common::environment::Environment;
use reqwest::{header, Client};
use tokio::fs::read_to_string;

/// # Errors
#[allow(clippy::module_name_repetitions)]
pub async fn create_client() -> Result<reqwest::Client, ApiError> {
    let environment = Environment::get();
    match environment.user_dot_grafbase_path.try_exists() {
        Ok(true) => {}
        Ok(false) => return Err(ApiError::NotLoggedIn),
        Err(error) => return Err(ApiError::ReadUserDotGrafbaseFolder(error)),
    }

    let credentials_file_path = environment.user_dot_grafbase_path.join(CREDENTIALS_FILE);

    match credentials_file_path.try_exists() {
        Ok(true) => {}
        Ok(false) => return Err(ApiError::NotLoggedIn),
        Err(error) => return Err(ApiError::ReadCredentialsFile(error)),
    }

    let credential_file = read_to_string(environment.user_dot_grafbase_path.join(CREDENTIALS_FILE))
        .await
        .map_err(ApiError::ReadCredentialsFile)?;

    let credentials: Credentials<'_> =
        serde_json::from_str(&credential_file).map_err(|_| ApiError::CorruptCredentialsFile)?;

    let token = credentials.access_token;

    let mut headers = HeaderMap::new();
    let mut bearer_token =
        HeaderValue::from_str(&format!("Bearer {token}")).map_err(|_| ApiError::CorruptCredentialsFile)?;
    bearer_token.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, bearer_token);
    let mut user_agent = HeaderValue::from_str(USER_AGENT).expect("must be visible ascii");
    user_agent.set_sensitive(true);
    headers.insert(header::USER_AGENT, user_agent);

    Ok(Client::builder()
        .default_headers(headers)
        .build()
        .expect("TLS is supported in all targets"))
}
