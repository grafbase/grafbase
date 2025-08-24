use crate::prepare::cached::query_plan::{
    generated::{QueryPartition, QueryPartitionId},
    prelude::*,
};
use schema::{CompositeType, CompositeTypeId};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ResponseObjectSetDefinition @meta(module: "response_object_set") @indexed(id_size: "u16", deduplicated: true) {
///   ty: CompositeType!
///   query_partition: [QueryPartition!]! @vec
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct ResponseObjectSetMetadataRecord {
    pub ty_id: CompositeTypeId,
    pub query_partition_ids: Vec<QueryPartitionId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct ResponseObjectSetId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub(crate) struct ResponseObjectSetMetadata<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(crate) id: ResponseObjectSetId,
}

impl std::ops::Deref for ResponseObjectSetMetadata<'_> {
    type Target = ResponseObjectSetMetadataRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> ResponseObjectSetMetadata<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a ResponseObjectSetMetadataRecord {
        &self.ctx.cached.query_plan[self.id]
    }
    pub(crate) fn ty(&self) -> CompositeType<'a> {
        self.ty_id.walk(self.ctx)
    }
    pub(crate) fn query_partition(&self) -> impl Iter<Item = QueryPartition<'a>> + 'a {
        self.as_ref().query_partition_ids.walk(self.ctx)
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for ResponseObjectSetId {
    type Walker<'w>
        = ResponseObjectSetMetadata<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ResponseObjectSetMetadata {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for ResponseObjectSetMetadata<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseObjectSetDefinition")
            .field("ty", &self.ty())
            .field("query_partition", &self.query_partition())
            .finish()
    }
}
