use id_newtypes::IdRange;
use operation::{Location, PositionedResponseKey, QueryPosition, ResponseKey};
use query_solver::TypeConditionSharedVecId;
use walker::Walk;

use crate::prepare::CachedOperationContext;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PartitionTypenameFieldRecord {
    pub type_condition_ids: IdRange<TypeConditionSharedVecId>,
    pub query_position: Option<QueryPosition>,
    pub response_key: ResponseKey,
    pub location: Location,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct PartitionTypenameFieldId(u16);

/// __typename field
#[derive(Clone, Copy)]
pub(crate) struct PartitionTypenameField<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(crate) id: PartitionTypenameFieldId,
}

impl std::ops::Deref for PartitionTypenameField<'_> {
    type Target = PartitionTypenameFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> PartitionTypenameField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a PartitionTypenameFieldRecord {
        &self.ctx.cached.query_plan[self.id]
    }
}

impl PartitionTypenameFieldRecord {
    pub(crate) fn key(&self) -> PositionedResponseKey {
        PositionedResponseKey {
            query_position: self.query_position,
            response_key: self.response_key,
        }
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for PartitionTypenameFieldId {
    type Walker<'w>
        = PartitionTypenameField<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        PartitionTypenameField {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for PartitionTypenameField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypenameField")
            .field("key", &self.response_key)
            .field("location", &self.location)
            .finish()
    }
}
