use futures_util::future::BoxFuture;
use runtime::authentication::LegacyToken;

use crate::LegacyAuthorizer;

pub struct AnonymousAuthorizer;

impl LegacyAuthorizer for AnonymousAuthorizer {
    fn get_access_token(&self, _headers: &http::HeaderMap) -> BoxFuture<'_, Option<LegacyToken>> {
        Box::pin(async { Some(LegacyToken::Anonymous) })
    }
}
