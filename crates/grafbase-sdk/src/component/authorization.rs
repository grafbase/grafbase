use crate::{
    types::{AuthorizeQueryOutput, Headers},
    wit,
};

use super::{Component, state};

impl wit::AuthorizationGuest for Component {
    fn authorize_query(
        event_queue: wit::EventQueue,
        ctx: wit::AuthenticatedRequestContext,
        subgraph_headers: wit::Headers,
        elements: wit::QueryElements,
    ) -> Result<wit::AuthorizationOutput, wit::ErrorResponse> {
        state::with_event_queue(event_queue, || {
            let subgraph_headers: Headers = subgraph_headers.into();
            state::extension()?
                .authorize_query(&(ctx.into()), &subgraph_headers, (&elements).into())
                .map(
                    |AuthorizeQueryOutput {
                         decisions,
                         context,
                         state,
                         additional_headers,
                     }| {
                        wit::AuthorizationOutput {
                            decisions: decisions.into(),
                            context,
                            state,
                            subgraph_headers: subgraph_headers.into(),
                            additional_headers: additional_headers.map(Into::into),
                        }
                    },
                )
                .map_err(Into::into)
        })
    }

    fn authorize_response(
        event_queue: wit::EventQueue,
        ctx: wit::AuthorizedOperationContext,
        state: Vec<u8>,
        elements: wit::ResponseElements,
    ) -> Result<wit::AuthorizationDecisions, wit::Error> {
        state::with_event_queue(event_queue, || {
            state::extension()?
                .authorize_response(&(ctx.into()), state, (&elements).into())
                .map(Into::into)
                .map_err(Into::into)
        })
    }
}
