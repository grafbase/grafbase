//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/solved_operation.graphql
use crate::operation::solve::model::prelude::*;
use schema::{Type, TypeRecord};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type VariableDefinition @meta(module: "variable") @indexed(id_size: "u16") {
///   name: String!
///   name_location: Location!
///   default_value_id: QueryInputValueId
///   ty: Type!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct VariableDefinitionRecord {
    pub name: String,
    pub name_location: Location,
    pub default_value_id: Option<QueryInputValueId>,
    pub ty_record: TypeRecord,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct VariableDefinitionId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub(crate) struct VariableDefinition<'a> {
    pub(in crate::operation::solve::model) ctx: SolvedOperationContext<'a>,
    pub(crate) id: VariableDefinitionId,
}

impl std::ops::Deref for VariableDefinition<'_> {
    type Target = VariableDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> VariableDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a VariableDefinitionRecord {
        &self.ctx.operation[self.id]
    }
    pub(crate) fn ty(&self) -> Type<'a> {
        self.ty_record.walk(self.ctx)
    }
}

impl<'a> Walk<SolvedOperationContext<'a>> for VariableDefinitionId {
    type Walker<'w> = VariableDefinition<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: impl Into<SolvedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        VariableDefinition {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for VariableDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VariableDefinition")
            .field("name", &self.name)
            .field("name_location", &self.name_location)
            .field("default_value_id", &self.default_value_id)
            .field("ty", &self.ty())
            .finish()
    }
}
