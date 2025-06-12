use crate::wit;

use super::{Component, state};

impl wit::AuthenticationGuest for Component {
    fn authenticate(context: wit::SharedContext, headers: wit::Headers) -> Result<wit::Token, wit::ErrorResponse> {
        state::with_context(context, || {
            let result = state::extension()
                .map_err(|err| wit::ErrorResponse {
                    status_code: 500,
                    errors: vec![err],
                })?
                .authenticate(&headers.into());

            result.map(Into::into).map_err(Into::into)
        })
    }
}
