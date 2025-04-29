use operation::{PositionedResponseKey, ResponseKey};
use schema::{EnumDefinitionId, ScalarType, Wrapping};
use walker::Walk;

use crate::prepare::{DataOrLookupField, DataOrLookupFieldId, OperationPlanContext, QueryErrorId};

use super::{ConcreteShapeId, PolymorphicShapeId};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct FieldShapeRecord {
    pub expected_key: ResponseKey,
    pub key: PositionedResponseKey,
    pub id: DataOrLookupFieldId,
    pub shape: Shape,
    pub wrapping: Wrapping,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct FieldShapeId(u32);

impl<'ctx> Walk<OperationPlanContext<'ctx>> for FieldShapeId {
    type Walker<'w>
        = FieldShape<'w>
    where
        'ctx: 'w;

    fn walk<'w>(self, ctx: impl Into<OperationPlanContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        FieldShape {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::ops::Deref for FieldShape<'_> {
    type Target = FieldShapeRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FieldShape<'a> {
    pub(super) ctx: OperationPlanContext<'a>,
    pub(super) id: FieldShapeId,
}

#[allow(unused)]
impl<'a> FieldShape<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a FieldShapeRecord {
        &self.ctx.cached.shapes[self.id]
    }

    pub(crate) fn partition_field(&self) -> DataOrLookupField<'a> {
        self.as_ref().id.walk(self.ctx)
    }

    pub(crate) fn error_ids(&self) -> impl Iterator<Item = QueryErrorId> + 'a {
        self.ctx
            .plan
            .query_modifications
            .field_shape_id_to_error_ids
            .find_all(self.id)
            .copied()
    }

    pub(crate) fn is_skipped(&self) -> bool {
        self.ctx.plan.query_modifications.skipped_field_shapes[self.id]
    }
}

impl std::fmt::Debug for FieldShape<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldShape").finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(crate) enum Shape {
    Scalar(ScalarType),
    Enum(EnumDefinitionId),
    Concrete(ConcreteShapeId),
    Polymorphic(PolymorphicShapeId),
}

impl Shape {
    pub(crate) fn as_concrete(&self) -> Option<ConcreteShapeId> {
        match self {
            Shape::Concrete(id) => Some(*id),
            _ => None,
        }
    }
}
