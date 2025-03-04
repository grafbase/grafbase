//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
mod authorized;
mod complexity_control;
mod deprecated;
mod extension;

use crate::{RequiresScopesDirective, RequiresScopesDirectiveId, prelude::*};
pub use authorized::*;
pub use complexity_control::*;
pub use deprecated::*;
pub use extension::*;
#[allow(unused_imports)]
use walker::{Iter, Walk};

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
///   | CostDirective
///   | ListSizeDirective
///   | ExtensionDirective
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSystemDirectiveId {
    Authenticated,
    Authorized(AuthorizedDirectiveId),
    Cost(CostDirectiveId),
    Deprecated(DeprecatedDirectiveRecord),
    Extension(ExtensionDirectiveId),
    ListSize(ListSizeDirectiveId),
    RequiresScopes(RequiresScopesDirectiveId),
}

impl std::fmt::Debug for TypeSystemDirectiveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeSystemDirectiveId::Authenticated => write!(f, "Authenticated"),
            TypeSystemDirectiveId::Authorized(variant) => variant.fmt(f),
            TypeSystemDirectiveId::Cost(variant) => variant.fmt(f),
            TypeSystemDirectiveId::Deprecated(variant) => variant.fmt(f),
            TypeSystemDirectiveId::Extension(variant) => variant.fmt(f),
            TypeSystemDirectiveId::ListSize(variant) => variant.fmt(f),
            TypeSystemDirectiveId::RequiresScopes(variant) => variant.fmt(f),
        }
    }
}

impl From<AuthorizedDirectiveId> for TypeSystemDirectiveId {
    fn from(value: AuthorizedDirectiveId) -> Self {
        TypeSystemDirectiveId::Authorized(value)
    }
}
impl From<CostDirectiveId> for TypeSystemDirectiveId {
    fn from(value: CostDirectiveId) -> Self {
        TypeSystemDirectiveId::Cost(value)
    }
}
impl From<DeprecatedDirectiveRecord> for TypeSystemDirectiveId {
    fn from(value: DeprecatedDirectiveRecord) -> Self {
        TypeSystemDirectiveId::Deprecated(value)
    }
}
impl From<ExtensionDirectiveId> for TypeSystemDirectiveId {
    fn from(value: ExtensionDirectiveId) -> Self {
        TypeSystemDirectiveId::Extension(value)
    }
}
impl From<ListSizeDirectiveId> for TypeSystemDirectiveId {
    fn from(value: ListSizeDirectiveId) -> Self {
        TypeSystemDirectiveId::ListSize(value)
    }
}
impl From<RequiresScopesDirectiveId> for TypeSystemDirectiveId {
    fn from(value: RequiresScopesDirectiveId) -> Self {
        TypeSystemDirectiveId::RequiresScopes(value)
    }
}

impl TypeSystemDirectiveId {
    pub fn is_authenticated(&self) -> bool {
        matches!(self, TypeSystemDirectiveId::Authenticated)
    }
    pub fn is_authorized(&self) -> bool {
        matches!(self, TypeSystemDirectiveId::Authorized(_))
    }
    pub fn as_authorized(&self) -> Option<AuthorizedDirectiveId> {
        match self {
            TypeSystemDirectiveId::Authorized(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_cost(&self) -> bool {
        matches!(self, TypeSystemDirectiveId::Cost(_))
    }
    pub fn as_cost(&self) -> Option<CostDirectiveId> {
        match self {
            TypeSystemDirectiveId::Cost(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_deprecated(&self) -> bool {
        matches!(self, TypeSystemDirectiveId::Deprecated(_))
    }
    pub fn as_deprecated(&self) -> Option<DeprecatedDirectiveRecord> {
        match self {
            TypeSystemDirectiveId::Deprecated(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_extension(&self) -> bool {
        matches!(self, TypeSystemDirectiveId::Extension(_))
    }
    pub fn as_extension(&self) -> Option<ExtensionDirectiveId> {
        match self {
            TypeSystemDirectiveId::Extension(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_list_size(&self) -> bool {
        matches!(self, TypeSystemDirectiveId::ListSize(_))
    }
    pub fn as_list_size(&self) -> Option<ListSizeDirectiveId> {
        match self {
            TypeSystemDirectiveId::ListSize(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_requires_scopes(&self) -> bool {
        matches!(self, TypeSystemDirectiveId::RequiresScopes(_))
    }
    pub fn as_requires_scopes(&self) -> Option<RequiresScopesDirectiveId> {
        match self {
            TypeSystemDirectiveId::RequiresScopes(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum TypeSystemDirective<'a> {
    Authenticated(&'a Schema),
    Authorized(AuthorizedDirective<'a>),
    Cost(CostDirective<'a>),
    Deprecated(DeprecatedDirective<'a>),
    Extension(ExtensionDirective<'a>),
    ListSize(ListSizeDirective<'a>),
    RequiresScopes(RequiresScopesDirective<'a>),
}

impl std::fmt::Debug for TypeSystemDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeSystemDirective::Authenticated(_) => write!(f, "Authenticated"),
            TypeSystemDirective::Authorized(variant) => variant.fmt(f),
            TypeSystemDirective::Cost(variant) => variant.fmt(f),
            TypeSystemDirective::Deprecated(variant) => variant.fmt(f),
            TypeSystemDirective::Extension(variant) => variant.fmt(f),
            TypeSystemDirective::ListSize(variant) => variant.fmt(f),
            TypeSystemDirective::RequiresScopes(variant) => variant.fmt(f),
        }
    }
}

impl<'a> From<AuthorizedDirective<'a>> for TypeSystemDirective<'a> {
    fn from(item: AuthorizedDirective<'a>) -> Self {
        TypeSystemDirective::Authorized(item)
    }
}
impl<'a> From<CostDirective<'a>> for TypeSystemDirective<'a> {
    fn from(item: CostDirective<'a>) -> Self {
        TypeSystemDirective::Cost(item)
    }
}
impl<'a> From<DeprecatedDirective<'a>> for TypeSystemDirective<'a> {
    fn from(item: DeprecatedDirective<'a>) -> Self {
        TypeSystemDirective::Deprecated(item)
    }
}
impl<'a> From<ExtensionDirective<'a>> for TypeSystemDirective<'a> {
    fn from(item: ExtensionDirective<'a>) -> Self {
        TypeSystemDirective::Extension(item)
    }
}
impl<'a> From<ListSizeDirective<'a>> for TypeSystemDirective<'a> {
    fn from(item: ListSizeDirective<'a>) -> Self {
        TypeSystemDirective::ListSize(item)
    }
}

impl<'a> Walk<&'a Schema> for TypeSystemDirectiveId {
    type Walker<'w>
        = TypeSystemDirective<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let schema: &'a Schema = schema.into();
        match self {
            TypeSystemDirectiveId::Authenticated => TypeSystemDirective::Authenticated(schema),
            TypeSystemDirectiveId::Authorized(id) => TypeSystemDirective::Authorized(id.walk(schema)),
            TypeSystemDirectiveId::Cost(id) => TypeSystemDirective::Cost(id.walk(schema)),
            TypeSystemDirectiveId::Deprecated(item) => TypeSystemDirective::Deprecated(item.walk(schema)),
            TypeSystemDirectiveId::Extension(id) => TypeSystemDirective::Extension(id.walk(schema)),
            TypeSystemDirectiveId::ListSize(id) => TypeSystemDirective::ListSize(id.walk(schema)),
            TypeSystemDirectiveId::RequiresScopes(id) => TypeSystemDirective::RequiresScopes(id.walk(schema)),
        }
    }
}

impl<'a> TypeSystemDirective<'a> {
    pub fn id(&self) -> TypeSystemDirectiveId {
        match self {
            TypeSystemDirective::Authenticated(_) => TypeSystemDirectiveId::Authenticated,
            TypeSystemDirective::Authorized(walker) => TypeSystemDirectiveId::Authorized(walker.id),
            TypeSystemDirective::Cost(walker) => TypeSystemDirectiveId::Cost(walker.id),
            TypeSystemDirective::Deprecated(walker) => TypeSystemDirectiveId::Deprecated(walker.item),
            TypeSystemDirective::Extension(walker) => TypeSystemDirectiveId::Extension(walker.id),
            TypeSystemDirective::ListSize(walker) => TypeSystemDirectiveId::ListSize(walker.id),
            TypeSystemDirective::RequiresScopes(walker) => TypeSystemDirectiveId::RequiresScopes(walker.id),
        }
    }
    pub fn is_authorized(&self) -> bool {
        matches!(self, TypeSystemDirective::Authorized(_))
    }
    pub fn as_authorized(&self) -> Option<AuthorizedDirective<'a>> {
        match self {
            TypeSystemDirective::Authorized(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_cost(&self) -> bool {
        matches!(self, TypeSystemDirective::Cost(_))
    }
    pub fn as_cost(&self) -> Option<CostDirective<'a>> {
        match self {
            TypeSystemDirective::Cost(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_deprecated(&self) -> bool {
        matches!(self, TypeSystemDirective::Deprecated(_))
    }
    pub fn as_deprecated(&self) -> Option<DeprecatedDirective<'a>> {
        match self {
            TypeSystemDirective::Deprecated(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_extension(&self) -> bool {
        matches!(self, TypeSystemDirective::Extension(_))
    }
    pub fn as_extension(&self) -> Option<ExtensionDirective<'a>> {
        match self {
            TypeSystemDirective::Extension(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_list_size(&self) -> bool {
        matches!(self, TypeSystemDirective::ListSize(_))
    }
    pub fn as_list_size(&self) -> Option<ListSizeDirective<'a>> {
        match self {
            TypeSystemDirective::ListSize(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_requires_scopes(&self) -> bool {
        matches!(self, TypeSystemDirective::RequiresScopes(_))
    }
    pub fn as_requires_scopes(&self) -> Option<RequiresScopesDirective<'a>> {
        match self {
            TypeSystemDirective::RequiresScopes(item) => Some(*item),
            _ => None,
        }
    }
}
