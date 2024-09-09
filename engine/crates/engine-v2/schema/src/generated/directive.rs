//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
mod authorized;
mod deprecated;

use crate::{prelude::*, RequiresScopesDirective, RequiresScopesDirectiveId};
pub use authorized::*;
pub use deprecated::*;
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union TypeSystemDirective
///   @id
///   @meta(module: "directive")
///   @variants(empty: ["Authenticated"], remove_suffix: "Directive") =
///   | DeprecatedDirective
///   | RequiresScopesDirective
///   | AuthorizedDirective
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSystemDirectiveId {
    Authenticated,
    Authorized(AuthorizedDirectiveId),
    Deprecated(DeprecatedDirectiveRecord),
    RequiresScopes(RequiresScopesDirectiveId),
}

impl std::fmt::Debug for TypeSystemDirectiveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeSystemDirectiveId::Authenticated => write!(f, "Authenticated"),
            TypeSystemDirectiveId::Authorized(variant) => variant.fmt(f),
            TypeSystemDirectiveId::Deprecated(variant) => variant.fmt(f),
            TypeSystemDirectiveId::RequiresScopes(variant) => variant.fmt(f),
        }
    }
}

impl From<AuthorizedDirectiveId> for TypeSystemDirectiveId {
    fn from(value: AuthorizedDirectiveId) -> Self {
        TypeSystemDirectiveId::Authorized(value)
    }
}
impl From<DeprecatedDirectiveRecord> for TypeSystemDirectiveId {
    fn from(value: DeprecatedDirectiveRecord) -> Self {
        TypeSystemDirectiveId::Deprecated(value)
    }
}
impl From<RequiresScopesDirectiveId> for TypeSystemDirectiveId {
    fn from(value: RequiresScopesDirectiveId) -> Self {
        TypeSystemDirectiveId::RequiresScopes(value)
    }
}

#[derive(Clone, Copy)]
pub enum TypeSystemDirective<'a> {
    Authenticated,
    Authorized(AuthorizedDirective<'a>),
    Deprecated(DeprecatedDirective<'a>),
    RequiresScopes(RequiresScopesDirective<'a>),
}

impl std::fmt::Debug for TypeSystemDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeSystemDirective::Authenticated => write!(f, "Authenticated"),
            TypeSystemDirective::Authorized(variant) => variant.fmt(f),
            TypeSystemDirective::Deprecated(variant) => variant.fmt(f),
            TypeSystemDirective::RequiresScopes(variant) => variant.fmt(f),
        }
    }
}

impl Walk<Schema> for TypeSystemDirectiveId {
    type Walker<'a> = TypeSystemDirective<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        match self {
            TypeSystemDirectiveId::Authenticated => TypeSystemDirective::Authenticated,
            TypeSystemDirectiveId::Authorized(id) => TypeSystemDirective::Authorized(id.walk(schema)),
            TypeSystemDirectiveId::Deprecated(item) => TypeSystemDirective::Deprecated(item.walk(schema)),
            TypeSystemDirectiveId::RequiresScopes(id) => TypeSystemDirective::RequiresScopes(id.walk(schema)),
        }
    }
}

impl TypeSystemDirective<'_> {
    pub fn id(&self) -> TypeSystemDirectiveId {
        match self {
            TypeSystemDirective::Authenticated => TypeSystemDirectiveId::Authenticated,
            TypeSystemDirective::Authorized(walker) => TypeSystemDirectiveId::Authorized(walker.id),
            TypeSystemDirective::Deprecated(walker) => TypeSystemDirectiveId::Deprecated(walker.item),
            TypeSystemDirective::RequiresScopes(walker) => TypeSystemDirectiveId::RequiresScopes(walker.id),
        }
    }
}
