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

    fn on_response(
        context: wit::SharedContext,
        status: u16,
        headers: wit::Headers,
        event_queue: wit::EventQueue,
    ) -> Result<(), String> {
        state::with_context(context, || {
            let status = http::StatusCode::from_u16(status)
                .expect("we converted this from http::StatusCode in the host, this cannot be invalid");

            let mut headers = GatewayHeaders::from(headers);

            state::extension()
                .map_err(|err| err.message)?
                .on_response(status, &mut headers, event_queue.into())
                .map_err(|err| err.0.message)
        })
    }
}
