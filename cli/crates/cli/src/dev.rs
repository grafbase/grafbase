use crate::output::report;
use crate::CliError;
use backend::server_api::start_server;
use backend::types::ServerMessage;
use common::utils::get_thread_panic_message;
use std::sync::Once;
use std::thread;

static READY: Once = Once::new();

/// cli wrapper for [`backend::server_api::start_server`]
///
/// # Errors
///
/// returns [`CliError::BackendError`] if the the local gateway returns an error
///
/// returns [`CliError::ServerPanic`] if the development server panics
pub fn dev(search: bool, watch: bool, external_port: u16, tracing: bool) -> Result<(), CliError> {
    trace!("attempting to start server");

    let (server_handle, receiver) =
        start_server(external_port, search, watch, tracing).map_err(CliError::BackendError)?;

    let reporter_handle = thread::spawn(move || {
        let mut resolvers_reported = false;

        while let Ok(message) = receiver.recv() {
            match message {
                ServerMessage::Ready(port) => {
                    READY.call_once(|| report::start_server(resolvers_reported, port, external_port));
                }
                ServerMessage::Reload(path) => report::reload(path),
                ServerMessage::StartResolverBuild(resolver_name) => {
                    report::start_resolver_build(&resolver_name);
                }
                ServerMessage::CompleteResolverBuild { name, duration } => {
                    resolvers_reported = true;
                    report::complete_resolver_build(&name, duration);
                }
                ServerMessage::ResolverMessage {
                    resolver_name,
                    message,
                    level,
                } => {
                    report::resolver_message(&resolver_name, &message, level);
                }
                ServerMessage::CompilationError(error) => report::error(&CliError::CompilationError(error)),
            }
        }
    });

    server_handle
        .join()
        .map_err(|parameter| match get_thread_panic_message(&parameter) {
            Some(message) => CliError::ServerPanic(message),
            None => CliError::ServerPanic("unknown error".to_owned()),
        })?
        .map_err(CliError::ServerError)?;

    reporter_handle.join().expect("cannot panic");

    Ok(())
}
