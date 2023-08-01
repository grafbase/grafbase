use crate::cli_input::LogLevelFilters;
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
#[allow(clippy::fn_params_excessive_bools)]
pub fn dev(
    search: bool,
    watch: bool,
    external_port: u16,
    log_level_filters: LogLevelFilters,
    tracing: bool,
) -> Result<(), CliError> {
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
                ServerMessage::StartUdfBuild { udf_kind, udf_name } => {
                    report::start_udf_build(udf_kind, &udf_name);
                }
                ServerMessage::CompleteUdfBuild {
                    udf_kind,
                    udf_name,
                    duration,
                } => {
                    resolvers_reported = true;
                    report::complete_udf_build(udf_kind, &udf_name, duration);
                }
                ServerMessage::UdfMessage {
                    udf_kind,
                    udf_name,
                    message,
                    level,
                } => {
                    report::udf_message(udf_kind, &udf_name, &message, level, log_level_filters.functions);
                }
                ServerMessage::OperationLogMessage { event_type, .. } => {
                    report::operation_log(event_type, log_level_filters.graphql_operations);
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
