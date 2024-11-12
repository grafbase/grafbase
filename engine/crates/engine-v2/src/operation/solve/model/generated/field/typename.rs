//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/solved_operation.graphql
use crate::operation::solve::model::prelude::*;
use schema::{CompositeType, CompositeTypeId};
use walker::Walk;

/// __typename field
///
/// --------------
/// Generated from:
///
/// ```custom,{.language-graphql}
/// type TypenameField @meta(module: "field/typename") @indexed(id_size: "u32") {
///   key: PositionedResponseKey!
///   location: Location!
///   type_condition: CompositeType!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct TypenameFieldRecord {
    pub key: PositionedResponseKey,
    pub location: Location,
    pub type_condition_id: CompositeTypeId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct TypenameFieldId(std::num::NonZero<u32>);

/// __typename field
#[derive(Clone, Copy)]
pub(crate) struct TypenameField<'a> {
    pub(in crate::operation::solve::model) ctx: SolvedOperationContext<'a>,
    pub(crate) id: TypenameFieldId,
}

impl std::ops::Deref for TypenameField<'_> {
    type Target = TypenameFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> TypenameField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a TypenameFieldRecord {
        &self.ctx.operation[self.id]
    }
    pub(crate) fn type_condition(&self) -> CompositeType<'a> {
        self.type_condition_id.walk(self.ctx)
    }
}

impl<'a> Walk<SolvedOperationContext<'a>> for TypenameFieldId {
    type Walker<'w> = TypenameField<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: impl Into<SolvedOperationContext<'a>>) -> Self::Walker<'w>
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
            .field("key", &self.key)
            .field("location", &self.location)
            .field("type_condition", &self.type_condition())
            .finish()
    }
}
