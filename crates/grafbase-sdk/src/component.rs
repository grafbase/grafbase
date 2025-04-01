mod authentication;
mod authorization;
mod error;
mod extension;
mod field_resolver;
mod state;

use crate::{
    types::Configuration,
    wit::{Error, ErrorResponse, Guest, SchemaDirective},
};

pub use error::SdkError;
pub(crate) use extension::AnyExtension;
pub(crate) use state::register_extension;

pub(crate) struct Component;

impl Guest for Component {
    fn init(directives: Vec<SchemaDirective>, configuration: Vec<u8>) -> Result<(), String> {
        let directives = directives.into_iter().map(Into::into).collect();
        let config = Configuration::new(configuration);
        state::init(directives, config).map_err(|e| e.to_string())
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
