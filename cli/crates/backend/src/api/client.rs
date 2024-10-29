use super::consts::GRAFBASE_ACCESS_TOKEN_ENV_VAR;
use super::types::Credentials;
use super::{consts::CREDENTIALS_FILE, errors::ApiError};
use common::consts::USER_AGENT;
use common::environment::Environment;
use reqwest::header::HeaderValue;
use reqwest::{header, Client};
use std::env;
use tokio::fs::read_to_string;

const CLIENT_NAME_HEADER: &str = "x-grafbase-client-name";
const CLIENT_VERSION_HEADER: &str = "x-grafbase-client-version";

/// # Errors
///
/// See [`ApiError`]
#[allow(clippy::module_name_repetitions)]
pub async fn create_client() -> Result<reqwest::Client, ApiError> {
    let token = get_access_token().await?;
    let mut headers = header::HeaderMap::new();

    let mut bearer_token =
        HeaderValue::from_str(&format!("Bearer {token}")).map_err(|_| ApiError::CorruptAccessToken)?;

    bearer_token.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, bearer_token);

    let mut user_agent = HeaderValue::from_str(USER_AGENT).expect("must be visible ascii");
    user_agent.set_sensitive(true);

    headers.insert(header::USER_AGENT, user_agent);
    headers.insert(CLIENT_NAME_HEADER, HeaderValue::from_static("Grafbase CLI"));

    headers.insert(
        CLIENT_VERSION_HEADER,
        HeaderValue::from_static(env!("CARGO_PKG_VERSION")),
    );

    Ok(Client::builder()
        .default_headers(headers)
        .build()
        .expect("TLS is supported in all targets"))
}

async fn get_access_token() -> Result<String, ApiError> {
    match get_access_token_from_file().await {
        Ok(token) => Ok(token),
        // attempt to also check GRAFBASE_ACCESS_TOKEN_ENV_VAR, returning the original error if it doesn't exist
        Err(error) => env::var(GRAFBASE_ACCESS_TOKEN_ENV_VAR).map_err(|_| error),
    }
}

async fn get_access_token_from_file() -> Result<String, ApiError> {
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

    Ok(credentials.access_token.to_owned())
}
