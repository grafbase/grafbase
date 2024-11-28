use std::num::NonZero;

use schema::ObjectDefinitionId;
use walker::Walk;

use crate::operation::OperationPlanContext;

use super::ConcreteShapeId;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PolymorphicShapeRecord {
    // Sorted by Object typename
    pub possibilities: Vec<(ObjectDefinitionId, ConcreteShapeId)>,
    pub fallback: Option<ConcreteShapeId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct PolymorphicShapeId(NonZero<u32>);

impl<'ctx> Walk<OperationPlanContext<'ctx>> for PolymorphicShapeId {
    type Walker<'w>
        = PolymorphicShape<'w>
    where
        'ctx: 'w;

    fn walk<'w>(self, ctx: impl Into<OperationPlanContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        PolymorphicShape {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::ops::Deref for PolymorphicShape<'_> {
    type Target = PolymorphicShapeRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

pub(crate) struct PolymorphicShape<'a> {
    pub(super) ctx: OperationPlanContext<'a>,
    pub(super) id: PolymorphicShapeId,
}

impl<'a> PolymorphicShape<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a PolymorphicShapeRecord {
        &self.ctx.solved_operation.shapes[self.id]
    }
}
