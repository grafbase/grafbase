use super::{Component, state};
use crate::wit;

impl wit::HooksGuest for Component {
    fn on_request(url: String, method: wit::HttpMethod, headers: wit::Headers) -> Result<(), wit::ErrorResponse> {
        state::extension()?
            .on_request(&url, method.into(), headers.into())
            .map_err(Into::into)
    }

    fn on_response(status: u16, headers: wit::Headers) -> Result<(), wit::ErrorResponse> {
        let status = http::StatusCode::from_u16(status)
            .expect("we converted this from http::StatusCode in the host, this cannot be invalid");

        state::extension()?
            .on_response(status, headers.into())
            .map_err(Into::into)
    }
}
