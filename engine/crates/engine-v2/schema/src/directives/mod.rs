mod authorized;
mod cache_control;
mod requires_scopes;

pub use authorized::*;
pub use cache_control::*;
pub use requires_scopes::*;

use crate::{AuthorizedDirectiveId, CacheControlId, RequiredScopesId, StringId};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum TypeSystemDirective {
    Deprecated(Deprecated),
    Authenticated,
    RequiresScopes(RequiredScopesId),
    CacheControl(CacheControlId),
    Authorized(AuthorizedDirectiveId),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Deprecated {
    pub reason: Option<StringId>,
}
