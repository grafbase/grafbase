mod cache_control;
mod requires_scopes;

pub use cache_control::*;
pub use requires_scopes::*;

use crate::{CacheControlId, RequiredScopesId, StringId};

#[derive(Debug)]
pub enum TypeSystemDirective {
    Deprecated(Deprecated),
    Authenticated,
    RequiresScopes(RequiredScopesId),
    CacheControl(CacheControlId),
}

#[derive(Debug)]
pub struct Deprecated {
    pub reason: Option<StringId>,
}
