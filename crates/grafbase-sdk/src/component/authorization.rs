use crate::{
    types::{AuthorizeQueryOutput, Headers},
    wit,
};

use super::{Component, state};

impl wit::AuthorizationGuest for Component {
    fn authorize_query(
        context: wit::SharedContext,
        headers: wit::Headers,
        elements: wit::QueryElements,
    ) -> Result<wit::AuthorizationOutput, wit::ErrorResponse> {
        state::with_context(context, || {
            let mut headers: Headers = headers.into();
            state::extension()?
                .authorize_query(&mut headers, (&elements).into())
                .map(
                    |AuthorizeQueryOutput {
                         decisions,
                         state,
                         extra_headers,
                     }| wit::AuthorizationOutput {
                        decisions: decisions.into(),
                        state,
                        subgraph_headers: headers.into(),
                        additional_headers: extra_headers.map(Into::into),
                    },
                )
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
