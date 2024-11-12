use std::num::NonZero;

use schema::ObjectDefinitionId;
use walker::Walk;

use crate::operation::OperationPlanContext;

use super::ConcreteObjectShapeId;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PolymorphicObjectShapeRecord {
    // Sorted by Object typename
    pub possibilities: Vec<(ObjectDefinitionId, ConcreteObjectShapeId)>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct PolymorphicObjectShapeId(NonZero<u32>);

impl<'ctx> Walk<OperationPlanContext<'ctx>> for PolymorphicObjectShapeId {
    type Walker<'w> = PolymorphicObjectShape<'w> where 'ctx: 'w;

    fn walk<'w>(self, ctx: impl Into<OperationPlanContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        PolymorphicObjectShape {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::ops::Deref for PolymorphicObjectShape<'_> {
    type Target = PolymorphicObjectShapeRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

pub(crate) struct PolymorphicObjectShape<'a> {
    pub(super) ctx: OperationPlanContext<'a>,
    pub(super) id: PolymorphicObjectShapeId,
}

impl<'a> PolymorphicObjectShape<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a PolymorphicObjectShapeRecord {
        &self.ctx.operation_solution.shapes[self.id]
    }
}
