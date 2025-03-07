use crate::wit::{AuthenticationGuest, Headers, Token};

use super::{state, Component};

impl AuthenticationGuest for Component {
    fn authenticate(headers: Headers) -> Result<Token, crate::wit::ErrorResponse> {
        let result = state::extension()
            .map_err(|err| crate::wit::ErrorResponse {
                status_code: 500,
                errors: vec![err],
            })?
            .authenticate(headers.into());

        result.map(Into::into).map_err(Into::into)
    }
}
