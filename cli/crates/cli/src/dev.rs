use crate::cli_input::LogLevelFilters;
use crate::output::report;
use crate::CliError;
use backend::types::{LogEventType, ServerMessage};
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
    const EXPIRY_TIME: tokio::time::Duration = tokio::time::Duration::from_secs(60);

    trace!("attempting to start server");
    let (message_sender, mut message_receiver) = tokio::sync::mpsc::unbounded_channel::<ServerMessage>();

    let server = server::start(external_port, search, watch, tracing, message_sender);
    let reporter = async move {
        let mut resolvers_reported = false;

        // We group messages by operation (request ID). Because messages come in as a stream of events,
        // we need to group them on the fly and “flush” as a tree only when the final operation completion
        // event is observed.
        let mut message_group_buffer = std::collections::HashMap::new();

        while let Some(message) = message_receiver.recv().await {
            match message {
                ServerMessage::Ready { port, .. } => {
                    READY.call_once(|| report::start_dev_server(resolvers_reported, port, external_port));
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
                ServerMessage::RequestScopedMessage { event_type, request_id } => match event_type {
                    LogEventType::RequestCompleted {
                        name,
                        duration,
                        request_completed_type,
                    } => {
                        let nested_events = message_group_buffer
                            .remove(&request_id)
                            .map(|(_, events)| events)
                            .unwrap_or_default();
                        report::operation_log(name, duration, request_completed_type, nested_events, log_level_filters);
                    }
                    LogEventType::NestedEvent(nested_event) => {
                        message_group_buffer
                            .entry(request_id)
                            .or_insert_with(|| (tokio::time::Instant::now(), vec![]))
                            .1
                            .push(nested_event);
                    }
                },
                ServerMessage::CompilationError(error) => report::error(&CliError::CompilationError(error)),
                ServerMessage::StartUdfBuildAll | ServerMessage::CompleteUdfBuildAll { .. } => {}
            }

            // Flush nested events that are really old – if a user interrupts a request, we will not see an operation completion event.
            message_group_buffer.retain(|_, (created, _)| created.elapsed() < EXPIRY_TIME);
        }
    };

    let handle = thread::spawn(move || {
        #[allow(clippy::ignored_unit_patterns)]
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            tokio::select! {
                result = server => {
                    result?;
                }
                _ = reporter => {}
            }
            Ok(())
        })
    });

    handle
        .join()
        .map_err(|parameter| match get_thread_panic_message(&parameter) {
            Some(message) => CliError::ServerPanic(message),
            None => CliError::ServerPanic("unknown error".to_owned()),
        })?
        .map_err(CliError::ServerError)?;

    Ok(())
}
