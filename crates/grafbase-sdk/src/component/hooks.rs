use super::{Component, state};
use crate::{types::GatewayHeaders, wit};

impl wit::HooksGuest for Component {
    fn on_request(url: String, method: wit::HttpMethod, headers: wit::Headers) -> Result<(), wit::ErrorResponse> {
        let mut headers = GatewayHeaders::from(headers);

        state::extension()?
            .on_request(&url, method.into(), &mut headers)
            .map_err(Into::into)
    }

    fn on_response(status: u16, headers: wit::Headers) -> Result<(), String> {
        let status = http::StatusCode::from_u16(status)
            .expect("we converted this from http::StatusCode in the host, this cannot be invalid");

        let mut headers = GatewayHeaders::from(headers);

        state::extension()
            .map_err(|e| e.message)?
            .on_response(status, &mut headers)
    }
}
