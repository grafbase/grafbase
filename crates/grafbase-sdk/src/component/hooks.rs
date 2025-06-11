use super::{Component, state};
use crate::{types::GatewayHeaders, wit};

impl wit::HooksGuest for Component {
    fn on_request(
        context: wit::SharedContext,
        url: String,
        method: wit::HttpMethod,
        headers: wit::Headers,
    ) -> Result<(), wit::ErrorResponse> {
        state::with_context(context, || {
            let mut headers = GatewayHeaders::from(headers);

            state::extension()?
                .on_request(&url, method.into(), &mut headers)
                .map_err(Into::into)
        })
    }

    fn on_response(context: wit::SharedContext, status: u16, headers: wit::Headers) -> Result<(), String> {
        state::with_context(context, || {
            let status = http::StatusCode::from_u16(status)
                .expect("we converted this from http::StatusCode in the host, this cannot be invalid");

            let mut headers = GatewayHeaders::from(headers);
            let event_queue = state::current_context().event_queue().into();

            state::extension()
                .map_err(|e| e.message)?
                .on_response(status, &mut headers, event_queue)
        })
    }
}
