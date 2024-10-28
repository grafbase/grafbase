use super::errors::{ApiError, LoginApiError};
use super::types::LoginMessage;
use axum::{
    extract::{Query, State},
    response::Redirect,
    routing::get,
    Router,
};
use common::consts::CREDENTIALS_FILE;
use common::environment::{Credentials, Environment, PlatformData};
use serde::Deserialize;
use std::{fs::create_dir_all, net::Ipv4Addr, path::PathBuf, sync::mpsc::Sender as MspcSender};
use tokio::net::TcpListener;
use tokio::sync::mpsc::Sender;
use tower_http::trace::TraceLayer;
use urlencoding::encode;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenQueryParams {
    token: String,
}

async fn token<'a>(
    State(LoginApiState {
        api_url,
        auth_url,
        shutdown_sender,
        user_dot_grafbase_path,
    }): State<LoginApiState>,
    query: Query<TokenQueryParams>,
) -> Result<Redirect, Redirect> {
    let access_token = query.token.clone();
    let credentials_path = user_dot_grafbase_path.join(CREDENTIALS_FILE);
    let write_result = tokio::fs::write(&credentials_path, Credentials::new(access_token, api_url).to_string()).await;

    if write_result.is_ok() {
        // the current connection will still be redirected before closing the server
        shutdown_sender.send(Ok(())).await.expect("must be open");
        Ok(Redirect::temporary(&format!("{auth_url}?success=true")))
    } else {
        // the current connection will still be redirected before closing the server
        shutdown_sender
            .send(Err(LoginApiError::WriteCredentialFile(credentials_path)))
            .await
            .expect("must be open");
        Err(Redirect::temporary(&format!(
            "{auth_url}?success=false&error={}",
            encode("Could not write ~/.grafbase/credentials.json")
        )))
    }
}

#[derive(Clone)]
struct LoginApiState {
    shutdown_sender: Sender<Result<(), LoginApiError>>,
    user_dot_grafbase_path: PathBuf,
    auth_url: String,
    api_url: String,
}

/// Logs a user in via a browser flow
///
/// # Errors
///
/// - returns [`BackendError::FindUserDotGrafbaseFolder`] if the path of ~/.grafbase could not be found
///
/// - returns [`BackendError::CreateUserDotGrafbaseFolder`] if ~/.grafbase could not be created
///
/// - returns [`BackendError::ReadUserDotGrafbaseFolder`] if ~/.grafbase could not be read
///
/// - returns [`BackendError::StartLoginServer`] if the login server could not be started
#[allow(clippy::needless_pass_by_value)] // &Sender is not Sync
#[tokio::main]
pub async fn login(message_sender: MspcSender<LoginMessage>) -> Result<(), ApiError> {
    let environment = Environment::get();
    let platform_data = PlatformData::get();

    match environment.user_dot_grafbase_path.try_exists() {
        Ok(true) => {}
        Ok(false) => {
            create_dir_all(&environment.user_dot_grafbase_path).map_err(ApiError::CreateUserDotGrafbaseFolder)?;
        }
        Err(error) => return Err(ApiError::ReadUserDotGrafbaseFolder(error)),
    }

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|_| ApiError::FindAvailablePort)?;

    let port = listener.local_addr().map_err(|_| ApiError::FindAvailablePort)?.port();

    let auth_url = platform_data.auth_url.clone();
    let url = &format!("{auth_url}?callback={}", encode(&format!("http://127.0.0.1:{port}")));

    message_sender
        .send(LoginMessage::CallbackUrl(url.clone()))
        .expect("must be open");

    let (shutdown_sender, mut shutdown_receiver) = tokio::sync::mpsc::channel::<Result<(), LoginApiError>>(2);

    let router = Router::new()
        .route("/", get(token))
        .layer(TraceLayer::new_for_http())
        .with_state(LoginApiState {
            api_url: platform_data.api_url.clone(),
            auth_url,
            shutdown_sender,
            user_dot_grafbase_path: environment.user_dot_grafbase_path.clone(),
        });

    let server = axum::serve(listener, router).with_graceful_shutdown(async move {
        let shutdown_result = shutdown_receiver.recv().await.expect("must be open");

        match shutdown_result {
            Ok(()) => message_sender.send(LoginMessage::Done).expect("must be open"),
            Err(error) => message_sender.send(LoginMessage::Error(error)).expect("must be open"),
        }
    });

    server.await.map_err(|_| ApiError::StartLoginServer)
}
