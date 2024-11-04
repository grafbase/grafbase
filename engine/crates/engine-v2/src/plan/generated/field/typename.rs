//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
use crate::plan::prelude::*;
use schema::{CompositeType, CompositeTypeId};
use walker::Walk;

/// __typename field
///
/// --------------
/// Generated from:
///
/// ```custom,{.language-graphql}
/// type TypenameField @meta(module: "field/typename") {
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

/// __typename field
#[derive(Clone, Copy)]
pub(crate) struct TypenameField<'a> {
    pub(in crate::plan) ctx: PlanContext<'a>,
    pub(in crate::plan) ref_: &'a TypenameFieldRecord,
}

impl std::ops::Deref for TypenameField<'_> {
    type Target = TypenameFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

#[allow(unused)]
impl<'a> TypenameField<'a> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a TypenameFieldRecord {
        self.ref_
    }
    pub(crate) fn type_condition(&self) -> CompositeType<'a> {
        self.type_condition_id.walk(self.ctx.schema)
    }
}

impl<'a> Walk<PlanContext<'a>> for &TypenameFieldRecord {
    type Walker<'w> = TypenameField<'w> where Self : 'w , 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        TypenameField { ctx, ref_: self }
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
