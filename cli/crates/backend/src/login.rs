use crate::{
    consts::{AUTH_URL, CREDENTIALS_FILE},
    errors::{BackendError, LoginApiError},
    types::{Credentials, LoginMessage},
};
use axum::{
    extract::{Query, State},
    response::Redirect,
    routing::get,
    Router,
};
use common::{
    consts::EPHEMERAL_PORT_RANGE, environment::get_user_dot_grafbase_path, types::LocalAddressType,
    utils::find_available_port_in_range,
};
use serde::Deserialize;
use std::{
    fs::create_dir_all,
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::mpsc::Sender as MspcSender,
};
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
        shutdown_sender,
        user_dot_grafbase_path,
    }): State<LoginApiState>,
    query: Query<TokenQueryParams>,
) -> Result<Redirect, Redirect> {
    let access_token = &query.token;

    let credentials_path = user_dot_grafbase_path.join(CREDENTIALS_FILE);

    let write_result = tokio::fs::write(&credentials_path, Credentials { access_token }.to_string()).await;

    if write_result.is_ok() {
        // the current connection will still be redirected before closing the server
        shutdown_sender.send(Ok(())).await.expect("must be open");
        Ok(Redirect::temporary(&format!("{AUTH_URL}?success=true")))
    } else {
        // the current connection will still be redirected before closing the server
        shutdown_sender
            .send(Err(LoginApiError::WriteCredentialFile(credentials_path)))
            .await
            .expect("must be open");
        Err(Redirect::temporary(&format!(
            "{AUTH_URL}?success=false&error={}",
            encode("Could not write ~/.grafbase/credentials.json")
        )))
    }
}

#[derive(Clone)]
struct LoginApiState {
    shutdown_sender: Sender<Result<(), LoginApiError>>,
    user_dot_grafbase_path: PathBuf,
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
pub async fn login(message_sender: MspcSender<LoginMessage>) -> Result<(), BackendError> {
    let user_dot_grafbase_path = get_user_dot_grafbase_path().ok_or(BackendError::FindUserDotGrafbaseFolder)?;

    match user_dot_grafbase_path.try_exists() {
        Ok(true) => {}
        Ok(false) => create_dir_all(&user_dot_grafbase_path).map_err(BackendError::CreateUserDotGrafbaseFolder)?,
        Err(error) => return Err(BackendError::ReadUserDotGrafbaseFolder(error)),
    }

    let port = find_available_port_in_range(EPHEMERAL_PORT_RANGE, LocalAddressType::Localhost)
        .ok_or(BackendError::FindAvailablePort)?;

    let url = &format!("{AUTH_URL}?callback={}", encode(&format!("http://127.0.0.1:{port}")));

    message_sender
        .send(LoginMessage::CallbackUrl(url.clone()))
        .expect("must be open");

    let (shutdown_sender, mut shutdown_receiver) = tokio::sync::mpsc::channel::<Result<(), LoginApiError>>(2);

    let router = Router::new()
        .route("/", get(token))
        .layer(TraceLayer::new_for_http())
        .with_state(LoginApiState {
            shutdown_sender,
            user_dot_grafbase_path,
        });

    let socket_address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    let server = axum::Server::bind(&socket_address)
        .serve(router.into_make_service())
        .with_graceful_shutdown(async {
            let shutdown_result = shutdown_receiver.recv().await.expect("must be open");

            match shutdown_result {
                Ok(_) => message_sender.send(LoginMessage::Done).expect("must be open"),
                Err(error) => message_sender.send(LoginMessage::Error(error)).expect("must be open"),
            }
        });

    server.await.map_err(|_| BackendError::StartLoginServer)?;

    Ok(())
}
