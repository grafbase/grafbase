use crate::{
    extension::resolver::Subscription,
    host::{AuthorizationContext, Headers},
    types::{
        AuthorizationDecisions, Error, ErrorResponse, FieldDefinitionDirective, FieldInputs, FieldOutput,
        QueryElements, Token,
    },
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
        inputs: FieldInputs<'_>,
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
        headers: Headers,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
    ) -> Result<Option<Vec<u8>>, Error> {
        Err("Resolver extension not initialized correctly.".into())
    }

    fn authorize_query(
        &mut self,
        ctx: AuthorizationContext,
        elements: QueryElements<'_>,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        Err(ErrorResponse::internal_server_error(
            "Authorization extension not initialized correctly.",
        ))
    }
}
