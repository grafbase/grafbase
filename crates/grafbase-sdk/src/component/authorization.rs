use crate::wit::{AuthorizationContext, AuthorizationDecisions, AuthorizationGuest, ErrorResponse, QueryElements};

use super::{state, Component};

impl AuthorizationGuest for Component {
    fn authorize_query(
        ctx: AuthorizationContext,
        elements: QueryElements,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        state::extension()?
            .authorize_query(ctx.into(), (&elements).into())
            .map(Into::into)
            .map_err(Into::into)
    }
}
