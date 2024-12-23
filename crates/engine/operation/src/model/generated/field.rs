//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/operation.graphql
mod argument;
mod data;
mod typename;

use crate::model::prelude::*;
pub use argument::*;
pub use data::*;
pub use typename::*;
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union Field @id @meta(module: "field") @variants(remove_suffix: true) = DataField | TypenameField
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FieldId {
    Data(DataFieldId),
    Typename(TypenameFieldId),
}

impl std::fmt::Debug for FieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldId::Data(variant) => variant.fmt(f),
            FieldId::Typename(variant) => variant.fmt(f),
        }
    }
}

impl From<DataFieldId> for FieldId {
    fn from(value: DataFieldId) -> Self {
        FieldId::Data(value)
    }
}
impl From<TypenameFieldId> for FieldId {
    fn from(value: TypenameFieldId) -> Self {
        FieldId::Typename(value)
    }
}

impl FieldId {
    pub fn is_data(&self) -> bool {
        matches!(self, FieldId::Data(_))
    }
    pub fn as_data(&self) -> Option<DataFieldId> {
        match self {
            FieldId::Data(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_typename(&self) -> bool {
        matches!(self, FieldId::Typename(_))
    }
    pub fn as_typename(&self) -> Option<TypenameFieldId> {
        match self {
            FieldId::Typename(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Field<'a> {
    Data(DataField<'a>),
    Typename(TypenameField<'a>),
}

impl std::fmt::Debug for Field<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Field::Data(variant) => variant.fmt(f),
            Field::Typename(variant) => variant.fmt(f),
        }
    }
}

impl<'a> From<DataField<'a>> for Field<'a> {
    fn from(item: DataField<'a>) -> Self {
        Field::Data(item)
    }
}
impl<'a> From<TypenameField<'a>> for Field<'a> {
    fn from(item: TypenameField<'a>) -> Self {
        Field::Typename(item)
    }
}

impl<'a> Walk<OperationContext<'a>> for FieldId {
    type Walker<'w>
        = Field<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let ctx: OperationContext<'a> = ctx.into();
        match self {
            FieldId::Data(id) => Field::Data(id.walk(ctx)),
            FieldId::Typename(id) => Field::Typename(id.walk(ctx)),
        }
    }
}

impl<'a> Field<'a> {
    pub fn id(&self) -> FieldId {
        match self {
            Field::Data(walker) => FieldId::Data(walker.id),
            Field::Typename(walker) => FieldId::Typename(walker.id),
        }
    }
    pub fn is_data(&self) -> bool {
        matches!(self, Field::Data(_))
    }
    pub fn as_data(&self) -> Option<DataField<'a>> {
        match self {
            Field::Data(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_typename(&self) -> bool {
        matches!(self, Field::Typename(_))
    }
    pub fn as_typename(&self) -> Option<TypenameField<'a>> {
        match self {
            Field::Typename(item) => Some(*item),
            _ => None,
        }
    }
}
