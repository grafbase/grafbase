use crate::{
    extension::resolver::Subscription,
    types::{
        AuthorizationDecisions, Error, ErrorResponse, FieldDefinitionDirective, FieldInputs, FieldOutput,
        QueryElements, Token,
    },
    wit::Headers,
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
        headers: Headers,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
        inputs: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        Err("Resolver extension not initialized correctly.".into())
    }

    fn resolve_subscription(
        &mut self,
        headers: Headers,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
    ) -> Result<Box<dyn Subscription>, Error> {
        Err("Resolver extension not initialized correctly.".into())
    }

    fn subscription_key(
        &mut self,
        headers: &Headers,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
    ) -> Result<Option<Vec<u8>>, Error> {
        Err("Resolver extension not initialized correctly.".into())
    }

    fn authorize_query<'a>(&'a mut self, elements: QueryElements<'a>) -> Result<AuthorizationDecisions, ErrorResponse> {
        Err(ErrorResponse::internal_server_error(
            "Authorization extension not initialized correctly.",
        ))
    }
}
