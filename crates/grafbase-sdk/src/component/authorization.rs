use crate::wit::{
    AuthorizationContext, AuthorizationDecisions, AuthorizationGuest, Error, ErrorResponse, QueryElements,
    ResponseElements,
};

use super::{state, Component};

impl AuthorizationGuest for Component {
    fn authorize_query(
        ctx: AuthorizationContext,
        elements: QueryElements,
    ) -> Result<(AuthorizationDecisions, Vec<u8>), ErrorResponse> {
        state::extension()?
            .authorize_query(ctx.into(), (&elements).into())
            .map(|(decisions, state)| (decisions.into(), state))
            .map_err(Into::into)
    }

    fn authorize_response(
        ctx: AuthorizationContext,
        state: Vec<u8>,
        elements: ResponseElements,
    ) -> Result<AuthorizationDecisions, Error> {
        state::extension()?
            .authorize_response(ctx.into(), state, (&elements).into())
            .map(Into::into)
            .map_err(Into::into)
    }
}
