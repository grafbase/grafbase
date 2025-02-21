//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/operation.graphql
use crate::model::{
    Location, ResponseKey,
    generated::{ExecutableDirective, ExecutableDirectiveId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// __typename field
///
/// --------------
/// Generated from:
///
/// ```custom,{.language-graphql}
/// type TypenameField @meta(module: "field/typename") @indexed(id_size: "u16") {
///   response_key: ResponseKey!
///   location: Location!
///   directives: [ExecutableDirective!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TypenameFieldRecord {
    pub response_key: ResponseKey,
    pub location: Location,
    pub directive_ids: Vec<ExecutableDirectiveId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct TypenameFieldId(std::num::NonZero<u16>);

/// __typename field
#[derive(Clone, Copy)]
pub struct TypenameField<'a> {
    pub(in crate::model) ctx: OperationContext<'a>,
    pub id: TypenameFieldId,
}

impl std::ops::Deref for TypenameField<'_> {
    type Target = TypenameFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> TypenameField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a TypenameFieldRecord {
        &self.ctx.operation[self.id]
    }
    pub fn directives(&self) -> impl Iter<Item = ExecutableDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.ctx)
    }
}

impl<'a> Walk<OperationContext<'a>> for TypenameFieldId {
    type Walker<'w>
        = TypenameField<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        TypenameField {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for TypenameField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypenameField")
            .field("response_key", &self.response_key)
            .field("location", &self.location)
            .field("directives", &self.directives())
            .finish()
    }
}
