//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/operation.graphql
use crate::model::{Location, QueryInputValueId, prelude::*};
use schema::{
    InputObjectDefinition, InputObjectDefinitionId, InputValueDefinition, InputValueDefinitionId, Type, TypeRecord,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type VariableDefinition @meta(module: "variable") @indexed(id_size: "u16") {
///   name: String!
///   name_location: Location!
///   default_value_id: QueryInputValueId
///   ty: Type!
///   one_of_input_field_usage: OneOfInputField
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VariableDefinitionRecord {
    pub name: String,
    pub name_location: Location,
    pub default_value_id: Option<QueryInputValueId>,
    pub ty_record: TypeRecord,
    pub one_of_input_field_usage_record: Option<OneOfInputFieldRecord>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct VariableDefinitionId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub struct VariableDefinition<'a> {
    pub(in crate::model) ctx: OperationContext<'a>,
    pub id: VariableDefinitionId,
}

impl std::ops::Deref for VariableDefinition<'_> {
    type Target = VariableDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> VariableDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a VariableDefinitionRecord {
        &self.ctx.operation[self.id]
    }
    pub fn ty(&self) -> Type<'a> {
        self.ty_record.walk(self.ctx)
    }
    pub fn one_of_input_field_usage(&self) -> Option<OneOfInputField<'a>> {
        self.as_ref().one_of_input_field_usage_record.walk(self.ctx)
    }
}

impl<'a> Walk<OperationContext<'a>> for VariableDefinitionId {
    type Walker<'w>
        = VariableDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
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
            .field("one_of_input_field_usage", &self.one_of_input_field_usage())
            .finish()
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type OneOfInputField @meta(module: "variable") @copy {
///   object: InputObjectDefinition!
///   field: InputValueDefinition!
///   location: Location!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct OneOfInputFieldRecord {
    pub object_id: InputObjectDefinitionId,
    pub field_id: InputValueDefinitionId,
    pub location: Location,
}

#[derive(Clone, Copy)]
pub struct OneOfInputField<'a> {
    pub(in crate::model) ctx: OperationContext<'a>,
    pub(in crate::model) item: OneOfInputFieldRecord,
}

impl std::ops::Deref for OneOfInputField<'_> {
    type Target = OneOfInputFieldRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> OneOfInputField<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &OneOfInputFieldRecord {
        &self.item
    }
    pub fn object(&self) -> InputObjectDefinition<'a> {
        self.object_id.walk(self.ctx)
    }
    pub fn field(&self) -> InputValueDefinition<'a> {
        self.field_id.walk(self.ctx)
    }
}

impl<'a> Walk<OperationContext<'a>> for OneOfInputFieldRecord {
    type Walker<'w>
        = OneOfInputField<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        OneOfInputField {
            ctx: ctx.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for OneOfInputField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OneOfInputField")
            .field("object", &self.object())
            .field("field", &self.field())
            .field("location", &self.location)
            .finish()
    }
}
