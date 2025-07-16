//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/operation.graphql
use crate::model::{QueryInputValueId, prelude::*};
use schema::{InputValueDefinition, InputValueDefinitionId};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type FieldArgument @meta(module: "field/argument") @indexed(id_size: "u16") {
///   definition: InputValueDefinition!
///   value_id: QueryInputValueId!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct FieldArgumentRecord {
    pub definition_id: InputValueDefinitionId,
    pub value_id: QueryInputValueId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct FieldArgumentId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub struct FieldArgument<'a> {
    pub(in crate::model) ctx: OperationContext<'a>,
    pub id: FieldArgumentId,
}

impl std::ops::Deref for FieldArgument<'_> {
    type Target = FieldArgumentRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> FieldArgument<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a FieldArgumentRecord {
        &self.ctx.operation[self.id]
    }
    pub fn definition(&self) -> InputValueDefinition<'a> {
        self.definition_id.walk(self.ctx)
    }
}

impl<'a> Walk<OperationContext<'a>> for FieldArgumentId {
    type Walker<'w>
        = FieldArgument<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        FieldArgument {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for FieldArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldArgument")
            .field("definition", &self.definition())
            .field("value_id", &self.value_id)
            .finish()
    }
}
