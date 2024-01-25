use std::num::NonZeroUsize;

use futures_util::TryFutureExt;

use crate::{cli_input::LogLevelFilters, errors::CliError};

pub fn build(parallelism: NonZeroUsize, tracing: bool) -> Result<(), CliError> {
    trace!("attempting to build server");
    crate::start::run(LogLevelFilters::default(), |message_sender| {
        server::ProductionServer::build(message_sender, parallelism, tracing, None).map_ok(|_| ())
    })
}
