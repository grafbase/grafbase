mod error;
mod extension;
mod state;

use crate::{
    types::{Configuration, FieldInputs},
    wit::{
        AuthorizationDecisions, Error, ErrorResponse, FieldDefinitionDirective, FieldOutput, Guest, Headers,
        QueryElements, SchemaDirective, SharedContext, Token,
    },
};

pub use error::SdkError;
pub(crate) use extension::AnyExtension;
pub(crate) use state::register_extension;

pub(crate) struct Component;

impl Guest for Component {
    fn init_gateway_extension(directives: Vec<SchemaDirective>, configuration: Vec<u8>) -> Result<(), String> {
        let directives = directives.into_iter().map(Into::into).collect();
        let config = Configuration::new(configuration);
        state::init(directives, config).map_err(|e| e.to_string())
    }

    fn resolve_field(
        context: SharedContext,
        subgraph_name: String,
        directive: FieldDefinitionDirective,
        inputs: Vec<Vec<u8>>,
    ) -> Result<FieldOutput, Error> {
        let result =
            state::extension()?.resolve_field(context, &subgraph_name, (&directive).into(), FieldInputs::new(inputs));

        result.map(Into::into).map_err(Into::into)
    }

    fn resolve_subscription(
        context: SharedContext,
        subgraph_name: String,
        directive: FieldDefinitionDirective,
    ) -> Result<(), Error> {
        let subscription = state::extension()?.resolve_subscription(context, &subgraph_name, (&directive).into())?;

        state::set_subscription(subscription);

        Ok(())
    }

    fn resolve_next_subscription_item() -> Result<Option<FieldOutput>, Error> {
        Ok(state::subscription()?.next()?.map(Into::into))
    }

    fn authenticate(headers: Headers) -> Result<Token, crate::wit::ErrorResponse> {
        let result = state::extension()
            .map_err(|err| crate::wit::ErrorResponse {
                status_code: 500,
                errors: vec![err],
            })?
            .authenticate(headers);

        result.map(Into::into).map_err(Into::into)
    }

    fn authorize_query(
        context: SharedContext,
        elements: QueryElements,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        state::extension()?
            .authorize_query(context, (&elements).into())
            .map(Into::into)
            .map_err(Into::into)
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
