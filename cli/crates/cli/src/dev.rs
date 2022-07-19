use crate::output::report;
use crate::CliError;
use backend::server_api::start_server;
use backend::types::ServerMessage;
use common::consts::DEFAULT_PORT;
use common::environment::Environment;
use common::utils::get_thread_panic_message;
use std::sync::Once;
use std::thread::spawn;

static READY: Once = Once::new();

/// cli wrapper for [`backend::server_api::start_server`]
///
/// # Errors
///
/// returns [`CliError::BackendError`] if the the local gateway returns an error
///
/// returns [`CliError::ServerPanic`] if the development server panics
pub fn dev(search: bool, watch: bool, external_port: Option<u16>) -> Result<(), CliError> {
    trace!("attempting to start server");

    Environment::try_init().map_err(CliError::CommonError)?;

    let start_port = external_port.unwrap_or(DEFAULT_PORT);
    let server_handle = match start_server(external_port, search, watch) {
        Ok((handle, receiver)) => {
            spawn(move || loop {
                match receiver.recv() {
                    Ok(ServerMessage::Ready(port)) => READY.call_once(|| report::start_server(port, start_port)),
                    Ok(ServerMessage::Reload) => report::reload(),
                    Err(_) => break,
                }
            });

            handle
        }
        Err(error) => return Err(CliError::BackendError(error)),
    };

    server_handle
        .join()
        .map_err(|parameter| match get_thread_panic_message(&parameter) {
            Some(message) => CliError::ServerPanic(message),
            None => CliError::ServerPanic("unknown error".to_owned()),
        })?
        .map_err(CliError::ServerError)?;

    Ok(())
}
