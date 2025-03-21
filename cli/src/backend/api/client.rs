use super::errors::ApiError;
use crate::common::consts::USER_AGENT;
use crate::common::environment::{LoginState, PlatformData};
use reqwest::header::HeaderValue;
use reqwest::{Client, header};

const CLIENT_NAME_HEADER: &str = "x-grafbase-client-name";
const CLIENT_VERSION_HEADER: &str = "x-grafbase-client-version";

pub(crate) fn create_unauthenticated_client() -> Result<reqwest::Client, ApiError> {
    create_client_inner(None)
}

/// # Errors
///
/// See [`ApiError`]
pub(crate) fn create_client() -> Result<reqwest::Client, ApiError> {
    let LoginState::LoggedIn(ref credentials) = PlatformData::get().login_state else {
        return Err(ApiError::NotLoggedIn);
    };

    create_client_inner(Some(&credentials.access_token))
}

fn create_client_inner(access_token: Option<&str>) -> Result<reqwest::Client, ApiError> {
    let mut headers = header::HeaderMap::new();

    if let Some(token) = access_token {
        let mut bearer_token =
            HeaderValue::from_str(&format!("Bearer {token}")).map_err(|_| ApiError::CorruptAccessToken)?;

        bearer_token.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, bearer_token);
    }

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
