use futures_util::future::BoxFuture;
use runtime::auth::LegacyToken;

use crate::Authorizer;

pub struct AnonymousAuthorizer;

impl Authorizer for AnonymousAuthorizer {
    fn get_access_token(&self, _headers: &http::HeaderMap) -> BoxFuture<'_, Option<LegacyToken>> {
        Box::pin(async { Some(LegacyToken::Anonymous) })
    }
}
