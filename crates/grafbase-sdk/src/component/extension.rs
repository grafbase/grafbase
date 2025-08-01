use crate::{
    extension::resolver::SubscriptionCallback,
    host_io::event_queue::EventQueue,
    types::{
        AuthorizationDecisions, Contract, ContractDirective, Error, ErrorResponse, GraphqlSubgraph, Headers,
        HttpRequestParts, OnRequestOutput, PublicMetadataEndpoint, QueryElements, ResolvedField, Response,
        ResponseElements, Token, Variables,
    },
};

#[expect(unused_variables)]
pub(crate) trait AnyExtension {
    fn authenticate(&mut self, headers: &Headers) -> Result<Token, ErrorResponse> {
        Err(ErrorResponse::internal_server_error()
            .with_error("Authentication extension not initialized correctly. Is it defined with the appropriate type?"))
    }

    fn public_metadata(&mut self) -> Result<Vec<PublicMetadataEndpoint>, Error> {
        Err(Error::new(
            "Authentication extension not initialized correctly. Is it defined with the appropriate type?",
        ))
    }

    fn construct(
        &mut self,
        key: String,
        directives: Vec<ContractDirective<'_>>,
        subgraphs: Vec<GraphqlSubgraph>,
    ) -> Result<Contract, Error> {
        Err(Error::new(
            "Contracts extension not initialized correctly. Is it defined with the appropriate type?",
        ))
    }

    fn prepare(&mut self, field: ResolvedField<'_>) -> Result<Vec<u8>, Error> {
        Err(
            "Selection set resolver extension not initialized correctly. Is it defined with the appropriate type?"
                .into(),
        )
    }

    fn resolve(&mut self, prepared: &[u8], headers: Headers, variables: Variables) -> Response {
        Response::error("Resolver extension not initialized correctly. Is it defined with the appropriate type?")
    }

    fn resolve_subscription<'a>(
        &'a mut self,
        prepared: &'a [u8],
        headers: Headers,
        variables: Variables,
    ) -> Result<(Option<Vec<u8>>, SubscriptionCallback<'a>), Error> {
        Err("Resolver extension not initialized correctly. Is it defined with the appropriate type?".into())
    }

    fn authorize_query(
        &mut self,
        headers: &mut Headers,
        token: Token,
        elements: QueryElements<'_>,
    ) -> Result<(AuthorizationDecisions, Vec<u8>), ErrorResponse> {
        Err(ErrorResponse::internal_server_error()
            .with_error("Authorization extension not initialized correctly. Is it defined with the appropriate type?"))
    }

    fn authorize_response(
        &mut self,
        state: Vec<u8>,
        elements: ResponseElements<'_>,
    ) -> Result<AuthorizationDecisions, Error> {
        Err("Authorization extension not initialized correctly. Is it defined with the appropriate type?".into())
    }

    fn on_request(
        &mut self,
        url: &str,
        method: http::Method,
        headers: &mut Headers,
    ) -> Result<OnRequestOutput, ErrorResponse> {
        Err(ErrorResponse::internal_server_error()
            .with_error("Hooks extension not initialized correctly. Is it defined with the appropriate type?"))
    }

    fn on_response(
        &mut self,
        status: http::StatusCode,
        headers: &mut Headers,
        event_queue: EventQueue,
    ) -> Result<(), Error> {
        Err(Error::new(
            "Hooks extension not initialized correctly. Is it defined with the appropriate type?",
        ))
    }

    fn on_subgraph_request(&mut self, parts: &mut HttpRequestParts) -> Result<(), Error> {
        Err(Error::new(
            "Hooks extension not initialized correctly. Is it defined with the appropriate type?",
        ))
    }
}
