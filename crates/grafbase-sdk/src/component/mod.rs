mod extension;
mod state;

use crate::{
    types::{Configuration, FieldInputs},
    wit::{Directive, Error, FieldDefinition, FieldOutput, Guest, Headers, SharedContext, Token},
};

pub(crate) use extension::AnyExtension;
pub(crate) use state::register_extension;

pub(crate) struct Component;

impl Guest for Component {
    fn init_gateway_extension(directives: Vec<Directive>, configuration: Vec<u8>) -> Result<(), String> {
        let directives = directives.into_iter().map(Into::into).collect();
        let config = Configuration::new(configuration);
        state::init(directives, config).map_err(|e| e.to_string())
    }

    fn resolve_field(
        context: SharedContext,
        directive: Directive,
        definition: FieldDefinition,
        inputs: Vec<Vec<u8>>,
    ) -> Result<FieldOutput, Error> {
        let result =
            state::extension()?.resolve_field(context, directive.into(), definition.into(), FieldInputs::new(inputs));

        result.map(Into::into)
    }

    fn resolve_subscription(
        context: SharedContext,
        directive: Directive,
        definition: FieldDefinition,
    ) -> Result<(), Error> {
        let subscription = state::extension()?.resolve_subscription(context, directive.into(), definition.into())?;

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
}
