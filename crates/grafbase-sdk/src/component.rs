mod authentication;
mod authorization;
mod error;
mod extension;
mod field_resolver;
mod hooks;
mod selection_set_resolver;
mod state;

use crate::{
    types::Configuration,
    wit::{Error, ErrorResponse, EventQueue, Guest, Schema},
};

pub use error::SdkError;
pub(crate) use extension::AnyExtension;
pub(crate) use state::{queue_event, register_extension};

pub(crate) struct Component;

impl Guest for Component {
    fn init(
        subgraph_schemas: Vec<(String, Schema)>,
        configuration: Vec<u8>,
        event_queue: EventQueue,
    ) -> Result<(), String> {
        let config = Configuration::new(configuration);
        state::init(subgraph_schemas, config, event_queue).map_err(|e| e.to_string())
    }
}

impl From<Error> for ErrorResponse {
    fn from(err: Error) -> ErrorResponse {
        ErrorResponse {
            status_code: 500,
            errors: vec![err],
        }
    }
}
