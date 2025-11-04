use super::{Component, state};
use crate::{
    types::{Headers, HttpRequestParts},
    wit,
};

impl wit::HooksGuest for Component {
    fn on_request(
        event_queue: wit::EventQueue,
        parts: wit::HttpRequestParts,
    ) -> Result<wit::OnRequestOutput, wit::ErrorResponse> {
        state::with_event_queue(event_queue, || {
            let mut parts: HttpRequestParts = parts.into();

            state::extension()?
                .on_request(&parts.url, parts.method, &mut parts.headers)
                .map(|output| wit::OnRequestOutput {
                    contract_key: output.contract_key,
                    context: output.context,
                    headers: parts.headers.into(),
                })
                .map_err(Into::into)
        })
    }

    fn on_response(
        event_queue: wit::EventQueue,
        ctx: wit::RequestContext,
        status: u16,
        headers: wit::Headers,
    ) -> Result<wit::OnResponseOutput, String> {
        let mut status = http::StatusCode::from_u16(status)
            .expect("we converted this from http::StatusCode in the host, this cannot be invalid");

        let mut headers: Headers = headers.into();

        state::extension()
            .map_err(|err| err.message)?
            .on_response(&(ctx.into()), &mut status, &mut headers, event_queue.into())
            .map(|_| wit::OnResponseOutput {
                headers: headers.into(),
                status: status.as_u16(),
            })
            .map_err(|err| err.0.message)
    }

    fn on_virtual_subgraph_request(
        event_queue: wit::EventQueue,
        ctx: wit::AuthorizedOperationContext,
        subgraph_name: String,
        headers: wit::Headers,
    ) -> Result<wit::Headers, wit::Error> {
        state::with_event_queue(event_queue, || {
            let mut headers: Headers = headers.into();

            state::extension()?
                .on_virtual_subgraph_request(&(ctx.into()), &subgraph_name, &mut headers)
                .map(|_| headers.into())
                .map_err(Into::into)
        })
    }

    fn on_graphql_subgraph_request(
        event_queue: wit::EventQueue,
        ctx: wit::AuthorizedOperationContext,
        subgraph_name: String,
        parts: wit::HttpRequestParts,
    ) -> Result<wit::HttpRequestParts, wit::Error> {
        state::with_event_queue(event_queue, || {
            let mut parts: HttpRequestParts = parts.into();

            state::extension()?
                .on_graphql_subgraph_request(&(ctx.into()), &subgraph_name, &mut parts)
                .map(|_| parts.into())
                .map_err(Into::into)
        })
    }
}
