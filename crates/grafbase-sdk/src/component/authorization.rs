use crate::wit::{
    AuthorizationDecisions, AuthorizationGuest, Error, ErrorResponse, Headers, QueryElements, ResponseElements, Token,
};

use super::{Component, state};

impl AuthorizationGuest for Component {
    fn authorize_query(
        headers: Headers,
        token: Token,
        elements: QueryElements,
    ) -> Result<(AuthorizationDecisions, Vec<u8>), ErrorResponse> {
        state::extension()?
            .authorize_query(&mut headers.into(), token.into(), (&elements).into())
            .map(|(decisions, state)| (decisions.into(), state))
            .map_err(Into::into)
    }

    fn authorize_response(state: Vec<u8>, elements: ResponseElements) -> Result<AuthorizationDecisions, Error> {
        state::extension()?
            .authorize_response(state, (&elements).into())
            .map(Into::into)
            .map_err(Into::into)
    }
}
