//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/operation.graphql
mod fragment_spread;
mod inline_fragment;

use crate::model::{
    generated::{Field, FieldId},
    prelude::*,
};
pub use fragment_spread::*;
pub use inline_fragment::*;
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union Selection @id @meta(module: "selection") = Field | InlineFragment | FragmentSpread
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SelectionId {
    Field(FieldId),
    FragmentSpread(FragmentSpreadId),
    InlineFragment(InlineFragmentId),
}

impl std::fmt::Debug for SelectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectionId::Field(variant) => variant.fmt(f),
            SelectionId::FragmentSpread(variant) => variant.fmt(f),
            SelectionId::InlineFragment(variant) => variant.fmt(f),
        }
    }
}

impl From<FieldId> for SelectionId {
    fn from(value: FieldId) -> Self {
        SelectionId::Field(value)
    }
}
impl From<FragmentSpreadId> for SelectionId {
    fn from(value: FragmentSpreadId) -> Self {
        SelectionId::FragmentSpread(value)
    }
}
impl From<InlineFragmentId> for SelectionId {
    fn from(value: InlineFragmentId) -> Self {
        SelectionId::InlineFragment(value)
    }
}

impl SelectionId {
    pub fn is_field(&self) -> bool {
        matches!(self, SelectionId::Field(_))
    }
    pub fn as_field(&self) -> Option<&FieldId> {
        match self {
            SelectionId::Field(item) => Some(item),
            _ => None,
        }
    }
    pub fn is_fragment_spread(&self) -> bool {
        matches!(self, SelectionId::FragmentSpread(_))
    }
    pub fn as_fragment_spread(&self) -> Option<FragmentSpreadId> {
        match self {
            SelectionId::FragmentSpread(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_inline_fragment(&self) -> bool {
        matches!(self, SelectionId::InlineFragment(_))
    }
    pub fn as_inline_fragment(&self) -> Option<InlineFragmentId> {
        match self {
            SelectionId::InlineFragment(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Selection<'a> {
    Field(Field<'a>),
    FragmentSpread(FragmentSpread<'a>),
    InlineFragment(InlineFragment<'a>),
}

impl std::fmt::Debug for Selection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Selection::Field(variant) => variant.fmt(f),
            Selection::FragmentSpread(variant) => variant.fmt(f),
            Selection::InlineFragment(variant) => variant.fmt(f),
        }
    }
}

impl<'a> From<Field<'a>> for Selection<'a> {
    fn from(item: Field<'a>) -> Self {
        Selection::Field(item)
    }
}
impl<'a> From<FragmentSpread<'a>> for Selection<'a> {
    fn from(item: FragmentSpread<'a>) -> Self {
        Selection::FragmentSpread(item)
    }
}
impl<'a> From<InlineFragment<'a>> for Selection<'a> {
    fn from(item: InlineFragment<'a>) -> Self {
        Selection::InlineFragment(item)
    }
}

impl<'a> Walk<OperationContext<'a>> for SelectionId {
    type Walker<'w>
        = Selection<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let ctx: OperationContext<'a> = ctx.into();
        match self {
            SelectionId::Field(item) => Selection::Field(item.walk(ctx)),
            SelectionId::FragmentSpread(id) => Selection::FragmentSpread(id.walk(ctx)),
            SelectionId::InlineFragment(id) => Selection::InlineFragment(id.walk(ctx)),
        }
    }
}

impl<'a> Selection<'a> {
    pub fn id(&self) -> SelectionId {
        match self {
            Selection::Field(walker) => SelectionId::Field(walker.id()),
            Selection::FragmentSpread(walker) => SelectionId::FragmentSpread(walker.id),
            Selection::InlineFragment(walker) => SelectionId::InlineFragment(walker.id),
        }
    }
    pub fn is_field(&self) -> bool {
        matches!(self, Selection::Field(_))
    }
    pub fn as_field(&self) -> Option<Field<'a>> {
        match self {
            Selection::Field(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_fragment_spread(&self) -> bool {
        matches!(self, Selection::FragmentSpread(_))
    }
    pub fn as_fragment_spread(&self) -> Option<FragmentSpread<'a>> {
        match self {
            Selection::FragmentSpread(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_inline_fragment(&self) -> bool {
        matches!(self, Selection::InlineFragment(_))
    }
    pub fn as_inline_fragment(&self) -> Option<InlineFragment<'a>> {
        match self {
            Selection::InlineFragment(item) => Some(*item),
            _ => None,
        }
    }
}
