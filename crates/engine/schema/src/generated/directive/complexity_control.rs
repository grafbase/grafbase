//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{FieldDefinition, FieldDefinitionId, InputValueDefinition, InputValueDefinitionId},
    prelude::*,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type CostDirective @meta(module: "directive/complexity_control") @indexed(id_size: "u32") {
///   weight: Int!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CostDirectiveRecord {
    pub weight: i32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct CostDirectiveId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct CostDirective<'a> {
    pub(crate) schema: &'a Schema,
    pub id: CostDirectiveId,
}

impl std::ops::Deref for CostDirective<'_> {
    type Target = CostDirectiveRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> CostDirective<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a CostDirectiveRecord {
        &self.schema[self.id]
    }
}

impl<'a> Walk<&'a Schema> for CostDirectiveId {
    type Walker<'w> = CostDirective<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        CostDirective {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for CostDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CostDirective").field("weight", &self.weight).finish()
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ListSizeDirective @meta(module: "directive/complexity_control") @indexed(id_size: "u32") {
///   assumed_size: u32
///   slicing_arguments: [InputValueDefinition!]! @vec
///   sized_fields: [FieldDefinition!]! @vec
///   require_one_slicing_argument: Boolean!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ListSizeDirectiveRecord {
    pub assumed_size: Option<u32>,
    pub slicing_argument_ids: Vec<InputValueDefinitionId>,
    pub sized_field_ids: Vec<FieldDefinitionId>,
    pub require_one_slicing_argument: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ListSizeDirectiveId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct ListSizeDirective<'a> {
    pub(crate) schema: &'a Schema,
    pub id: ListSizeDirectiveId,
}

impl std::ops::Deref for ListSizeDirective<'_> {
    type Target = ListSizeDirectiveRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> ListSizeDirective<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a ListSizeDirectiveRecord {
        &self.schema[self.id]
    }
    pub fn slicing_arguments(&self) -> impl Iter<Item = InputValueDefinition<'a>> + 'a {
        self.as_ref().slicing_argument_ids.walk(self.schema)
    }
    pub fn sized_fields(&self) -> impl Iter<Item = FieldDefinition<'a>> + 'a {
        self.as_ref().sized_field_ids.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for ListSizeDirectiveId {
    type Walker<'w> = ListSizeDirective<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ListSizeDirective {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for ListSizeDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListSizeDirective")
            .field("assumed_size", &self.assumed_size)
            .field("slicing_arguments", &self.slicing_arguments())
            .field("sized_fields", &self.sized_fields())
            .field("require_one_slicing_argument", &self.require_one_slicing_argument)
            .finish()
    }
}
