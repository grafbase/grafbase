use crate::cli_input::LogLevelFilters;
use crate::output::report;
use crate::CliError;
use backend::types::{LogEventType, ServerMessage};
use common::utils::get_thread_panic_message;
use futures_util::Future;
use server::{errors::ServerError, types::NestedRequestScopedMessage};
use std::net::IpAddr;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::thread;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

struct MessageGroup {
    created_at: tokio::time::Instant,
    events: Vec<NestedRequestScopedMessage>,
}

impl MessageGroup {
    fn new() -> Self {
        Self {
            created_at: tokio::time::Instant::now(),
            events: vec![],
        }
    }
}

pub fn start(
    listen_address: IpAddr,
    port: u16,
    log_level_filters: LogLevelFilters,
    federated_graph_schema_path: Option<PathBuf>,
    tracing: bool,
) -> Result<(), CliError> {
    trace!("attempting to start server");
    run(log_level_filters, |message_sender| async move {
        // not sure we'll keep building in the start command, so keeping the same behavior as
        // before building UDFs serially.
        let parallelism = NonZeroUsize::new(1).expect("strictly positive");
        server::ProductionServer::build(message_sender, parallelism, tracing, federated_graph_schema_path)
            .await?
            .serve(listen_address, port)
            .await
    })
}

pub(crate) fn run<F>(
    log_level_filters: LogLevelFilters,
    build: impl FnOnce(UnboundedSender<ServerMessage>) -> F,
) -> Result<(), CliError>
where
    F: Future<Output = Result<(), ServerError>> + Send + 'static,
{
    let (message_sender, message_receiver) = tokio::sync::mpsc::unbounded_channel::<ServerMessage>();
    let action = build(message_sender);
    let log_reporter = log_reporter(message_receiver, log_level_filters);

    let handle = thread::spawn(move || {
        #[allow(clippy::ignored_unit_patterns)]
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            tokio::select! {
                result = action => {
                    result?;
                }
                _ = log_reporter => {}
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

async fn log_reporter(mut message_receiver: UnboundedReceiver<ServerMessage>, log_level_filters: LogLevelFilters) {
    const EVENT_MAX_DELAY: tokio::time::Duration = tokio::time::Duration::from_secs(60);

    // Using a LRU cache, we store data for at most the last 1024 requests. We'll certainly
    // revisit that logic, but it limits the possibility of memory problems.
    let mut message_group_buffer = lru::LruCache::new(NonZeroUsize::new(1024).unwrap());
    while let Some(message) = message_receiver.recv().await {
        #[allow(clippy::single_match)] // will certainly change in the future
        match message {
            ServerMessage::Ready {
                listen_address,
                port,
                is_federated: _,
            } => {
                report::start_prod_server(listen_address, port);
            }
            ServerMessage::RequestScopedMessage { event_type, request_id } => match event_type {
                LogEventType::RequestCompleted {
                    name,
                    duration,
                    request_completed_type,
                } => {
                    let nested_events = message_group_buffer
                        .pop(&request_id)
                        .map(|group: MessageGroup| group.events)
                        .unwrap_or_default();
                    report::operation_log(name, duration, request_completed_type, nested_events, log_level_filters);
                }
                LogEventType::NestedEvent(nested_event) => {
                    message_group_buffer
                        .get_or_insert_mut(request_id, MessageGroup::new)
                        .events
                        .push(nested_event);
                }
            },
            ServerMessage::StartUdfBuildAll => {
                report::start_udf_build_all();
            }
            ServerMessage::CompleteUdfBuildAll { duration } => {
                report::complete_udf_build_all(duration);
            }
            ServerMessage::CompilationError(error) => report::error(&CliError::CompilationError(error)),
            _ => {}
        }
        // Just avoiding keeping message indefinitely and imitating dev command logic.
        while message_group_buffer
            .peek_lru()
            .map(|(_, group)| group.created_at.elapsed() > EVENT_MAX_DELAY)
            .unwrap_or_default()
        {
            message_group_buffer.pop_lru();
        }
    }
}
