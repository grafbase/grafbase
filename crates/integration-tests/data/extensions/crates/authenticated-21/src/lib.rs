use grafbase_sdk::{
    AuthorizationExtension, IntoAuthorizeQueryOutput,
    types::{
        AuthenticatedRequestContext, AuthorizationDecisions, Configuration, Error, ErrorResponse, QueryElements,
        SubgraphHeaders,
    },
};

#[derive(AuthorizationExtension)]
struct Authenticated;

impl AuthorizationExtension for Authenticated {
    fn new(_: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    fn authorize_query(
        &mut self,
        ctx: &AuthenticatedRequestContext,
        _headers: &SubgraphHeaders,
        _elements: QueryElements<'_>,
    ) -> Result<impl IntoAuthorizeQueryOutput, ErrorResponse> {
        Ok(if ctx.token().is_anonymous() {
            AuthorizationDecisions::deny_all("Not authenticated")
        } else {
            AuthorizationDecisions::grant_all()
        })
    }
}
