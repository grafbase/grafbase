use crate::{types::Headers, wit};

use super::{Component, state};

impl wit::AuthenticationGuest for Component {
    fn authenticate(
        event_queue: wit::EventQueue,
        ctx: wit::RequestContext,
        headers: wit::Headers,
    ) -> Result<(wit::Headers, wit::Token), wit::ErrorResponse> {
        state::with_event_queue(event_queue, || {
            let headers: Headers = headers.into();
            let result = state::extension()
                .map_err(|err| wit::ErrorResponse {
                    status_code: 500,
                    errors: vec![err],
                    headers: None,
                })?
                .authenticate(&(ctx.into()), &headers);

            result.map(|token| (headers.into(), token.into())).map_err(Into::into)
        })
    }

    fn public_metadata() -> Result<Vec<wit::PublicMetadataEndpoint>, wit::Error> {
        state::extension()?
            .public_metadata()
            .map(|endpoints| endpoints.into_iter().map(|ep| ep.into()).collect())
            .map_err(From::from)
    }
}
