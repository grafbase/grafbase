use crate::{
    extension::field_resolver::Subscription,
    host_io::audit_logs::AuditLogs,
    types::{
        ArgumentValues, AuthorizationDecisions, Data, Error, ErrorResponse, Field, FieldDefinitionDirective,
        FieldInputs, FieldOutputs, GatewayHeaders, QueryElements, ResponseElements, SubgraphHeaders, Token,
    },
};

#[allow(unused_variables)]
pub(crate) trait AnyExtension {
    fn authenticate(&mut self, headers: &GatewayHeaders) -> Result<Token, ErrorResponse> {
        Err(ErrorResponse::internal_server_error().with_error("Authentication extension not initialized correctly."))
    }

    fn selection_set_resolver_prepare(&mut self, subgraph_name: &str, field: Field<'_>) -> Result<Vec<u8>, Error> {
        Err("Selection set resolver extension not initialized correctly.".into())
    }

    fn selection_set_resolver_resolve(
        &mut self,
        headers: SubgraphHeaders,
        subgraph_name: &str,
        prepared: Vec<u8>,
        arguments: ArgumentValues<'_>,
    ) -> Result<Data, Error> {
        Err("Selection set resolver extension not initialized correctly.".into())
    }

    fn resolve_field(
        &mut self,
        headers: SubgraphHeaders,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
        inputs: FieldInputs<'_>,
    ) -> Result<FieldOutputs, Error> {
        Err("Resolver extension not initialized correctly.".into())
    }

    fn resolve_subscription<'a>(
        &'a mut self,
        headers: SubgraphHeaders,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
    ) -> Result<Box<dyn Subscription + 'a>, Error> {
        Err("Resolver extension not initialized correctly.".into())
    }

    fn subscription_key(
        &mut self,
        headers: &SubgraphHeaders,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
    ) -> Result<Option<Vec<u8>>, Error> {
        Err("Resolver extension not initialized correctly.".into())
    }

    fn authorize_query(
        &mut self,
        headers: &mut SubgraphHeaders,
        token: Token,
        elements: QueryElements<'_>,
    ) -> Result<(AuthorizationDecisions, Vec<u8>), ErrorResponse> {
        Err(ErrorResponse::internal_server_error().with_error("Authorization extension not initialized correctly."))
    }

    fn authorize_response(
        &mut self,
        state: Vec<u8>,
        elements: ResponseElements<'_>,
    ) -> Result<AuthorizationDecisions, Error> {
        Err("Authorization extension not initialized correctly.".into())
    }

    fn on_request(
        &mut self,
        url: &str,
        method: http::Method,
        headers: &mut GatewayHeaders,
    ) -> Result<(), ErrorResponse> {
        Err(ErrorResponse::internal_server_error().with_error("Hooks extension not initialized correctly."))
    }

    fn on_response(
        &mut self,
        status: http::StatusCode,
        headers: &mut GatewayHeaders,
        audit_logs: AuditLogs,
    ) -> Result<(), String> {
        Err("Hooks extension not initialized correctly.".to_string())
    }
}
