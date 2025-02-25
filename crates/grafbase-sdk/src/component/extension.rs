use crate::{
    extension::{authorization::ResponseAuthorizer, resolver::Subscription},
    types::{
        Error, ErrorResponse, FieldDefinitionDirective, FieldInputs, FieldOutput, QueryAuthorization, QueryElements,
        Token,
    },
    wit::{Headers, SharedContext},
};

#[allow(unused_variables)]
pub(crate) trait AnyExtension {
    fn authenticate(&mut self, headers: Headers) -> Result<Token, ErrorResponse> {
        Err(ErrorResponse::internal_server_error(
            "Authentication extension not initialized correctly.",
        ))
    }

    fn resolve_field(
        &mut self,
        context: SharedContext,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
        inputs: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        Err("Resolver extension not initialized correctly.".into())
    }

    fn resolve_subscription(
        &mut self,
        context: SharedContext,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
    ) -> Result<Box<dyn Subscription>, Error> {
        Err("Resolver extension not initialized correctly.".into())
    }

    fn authorize_query<'a>(
        &'a mut self,
        context: SharedContext,
        elements: QueryElements<'a>,
    ) -> Result<QueryAuthorization<Box<dyn ResponseAuthorizer<'a>>>, ErrorResponse> {
        Err(ErrorResponse::internal_server_error(
            "Authorization extension not initialized correctly.",
        ))
    }
}
