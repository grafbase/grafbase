//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
use crate::plan::model::prelude::*;
use schema::{CompositeType, CompositeTypeId};
use walker::Walk;

/// __typename field
///
/// --------------
/// Generated from:
///
/// ```custom,{.language-graphql}
/// type TypenamePlanField @meta(module: "field/typename") @indexed(id_size: "u32") {
///   key: PositionedResponseKey!
///   location: Location!
///   type_condition: CompositeType!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct TypenamePlanFieldRecord {
    pub key: PositionedResponseKey,
    pub location: Location,
    pub type_condition_id: CompositeTypeId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct TypenamePlanFieldId(std::num::NonZero<u32>);

/// __typename field
#[derive(Clone, Copy)]
pub(crate) struct TypenamePlanField<'a> {
    pub(in crate::plan::model) ctx: PlanContext<'a>,
    pub(in crate::plan::model) id: TypenamePlanFieldId,
}

impl std::ops::Deref for TypenamePlanField<'_> {
    type Target = TypenamePlanFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> TypenamePlanField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a TypenamePlanFieldRecord {
        &self.ctx.operation_plan[self.id]
    }
    pub(crate) fn id(&self) -> TypenamePlanFieldId {
        self.id
    }
    pub(crate) fn type_condition(&self) -> CompositeType<'a> {
        self.type_condition_id.walk(self.ctx.schema)
    }
}

impl<'a> Walk<PlanContext<'a>> for TypenamePlanFieldId {
    type Walker<'w> = TypenamePlanField<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        TypenamePlanField { ctx, id: self }
    }
}

impl std::fmt::Debug for TypenamePlanField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypenamePlanField")
            .field("key", &self.key)
            .field("location", &self.location)
            .field("type_condition", &self.type_condition())
            .finish()
    }
}
