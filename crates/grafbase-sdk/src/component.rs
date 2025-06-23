mod authentication;
mod authorization;
mod error;
mod extension;
mod hooks;
mod resolver;
mod state;

use std::sync::atomic::Ordering;

use crate::{
    types::Configuration,
    wit::{Error, ErrorResponse, Guest, LogLevel, Schema},
};

pub use error::SdkError;
pub(crate) use extension::*;
pub(crate) use state::{can_skip_sending_events, queue_event, register_extension};

pub(crate) struct Component;

pub(crate) fn guest_log_level() -> u8 {
    state::GUEST_LOG_LEVEL.load(Ordering::Relaxed)
}

impl Guest for Component {
    fn init(
        subgraph_schemas: Vec<(String, Schema)>,
        configuration: Vec<u8>,
        can_skip_sending_events: bool,
        host_log_level: Option<LogLevel>,
    ) -> Result<(), String> {
        let config = Configuration::new(configuration);

        state::init(subgraph_schemas, config, can_skip_sending_events, host_log_level).map_err(|e| e.to_string())
    }
}

impl From<Error> for ErrorResponse {
    fn from(err: Error) -> ErrorResponse {
        ErrorResponse {
            status_code: 500,
            errors: vec![err],
            headers: None,
        }
    }
}
