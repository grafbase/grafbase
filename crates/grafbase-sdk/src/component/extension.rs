use crate::{
    extension::resolver::Subscription,
    types::{Directive, ErrorResponse, FieldDefinition, FieldInputs, FieldOutput, Token},
    wit::{Headers, SharedContext},
    Error,
};

#[allow(unused_variables)]
pub(crate) trait AnyExtension {
    fn authenticate(&mut self, headers: Headers) -> Result<Token, ErrorResponse> {
        Err(
            ErrorResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR).with_error(Error {
                extensions: Vec::new(),
                message: String::from("Is not an authentication extension."),
            }),
        )
    }

    fn resolve_field(
        &mut self,
        context: SharedContext,
        directive: Directive,
        definition: FieldDefinition,
        inputs: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        Err(Error {
            message: "Resolver extension not initialized correctly.".to_string(),
            extensions: Vec::new(),
        })
    }

    fn resolve_subscription(
        &mut self,
        context: SharedContext,
        directive: Directive,
        definition: FieldDefinition,
    ) -> Result<Box<dyn Subscription>, Error> {
        Err(Error {
            message: "Resolver extension not initialized correctly.".to_string(),
            extensions: Vec::new(),
        })
    }
}
