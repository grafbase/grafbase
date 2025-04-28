use id_newtypes::IdRange;
use operation::{Location, QueryPosition, ResponseKey};
use query_solver::TypeConditionSharedVecId;
use walker::Walk;

use crate::prepare::CachedOperationContext;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct TypenameFieldRecord {
    pub type_condition_ids: IdRange<TypeConditionSharedVecId>,
    pub query_position: Option<QueryPosition>,
    pub response_key: ResponseKey,
    pub location: Location,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct TypenameFieldId(u16);

/// __typename field
#[derive(Clone, Copy)]
pub(crate) struct TypenameField<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(crate) id: TypenameFieldId,
}

impl std::ops::Deref for TypenameField<'_> {
    type Target = TypenameFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> TypenameField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a TypenameFieldRecord {
        &self.ctx.cached.query_plan[self.id]
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for TypenameFieldId {
    type Walker<'w>
        = TypenameField<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        TypenameField {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for TypenameField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypenameField")
            .field("key", &self.response_key)
            .field("location", &self.location)
            .finish()
    }
}
