use crate::wit;

use super::{Component, state};

impl wit::AuthorizationGuest for Component {
    fn authorize_query(
        context: wit::SharedContext,
        headers: wit::Headers,
        token: wit::Token,
        elements: wit::QueryElements,
    ) -> Result<(wit::AuthorizationDecisions, Vec<u8>), wit::ErrorResponse> {
        state::with_context(context, || {
            state::extension()?
                .authorize_query(&mut headers.into(), token.into(), (&elements).into())
                .map(|(decisions, state)| (decisions.into(), state))
                .map_err(Into::into)
        })
    }

    fn authorize_response(
        context: wit::SharedContext,
        state: Vec<u8>,
        elements: wit::ResponseElements,
    ) -> Result<wit::AuthorizationDecisions, wit::Error> {
        state::with_context(context, || {
            state::extension()?
                .authorize_response(state, (&elements).into())
                .map(Into::into)
                .map_err(Into::into)
        })
    }
}
