use super::errors::ApiError;
use common::consts::USER_AGENT;
use common::environment::{LoginState, PlatformData};
use reqwest::header::HeaderValue;
use reqwest::{header, Client};

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

    Ok(Client::builder()
        .default_headers(headers)
        .build()
        .expect("TLS is supported in all targets"))
}

async fn get_access_token<'a>() -> Result<&'a str, ApiError> {
    let LoginState::LoggedIn(ref credentials) = PlatformData::get().login_state else {
        return Err(ApiError::NotLoggedIn);
    };
    Ok(&credentials.access_token)
}
