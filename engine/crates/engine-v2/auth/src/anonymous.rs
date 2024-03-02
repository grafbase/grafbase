use futures_util::future::BoxFuture;
use runtime::auth::AccessToken;

use crate::Authorizer;

pub struct AnonymousAuthorizer;

impl Authorizer for AnonymousAuthorizer {
    fn authorize(&self, _headers: &http::HeaderMap) -> BoxFuture<'_, Option<AccessToken>> {
        Box::pin(async { Some(AccessToken::Anonymous) })
    }
}
