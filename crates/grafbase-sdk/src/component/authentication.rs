use crate::wit::{AuthenticationGuest, ErrorResponse, Headers, Token};

use super::{state, Component};

impl AuthenticationGuest for Component {
    fn authenticate(headers: Headers) -> Result<Token, ErrorResponse> {
        let result = state::extension()
            .map_err(|err| ErrorResponse {
                status_code: 500,
                errors: vec![err],
            })?
            .authenticate(&headers.into());

        result.map(Into::into).map_err(Into::into)
    }
}
