use std::num::NonZero;

use schema::{EnumDefinitionId, FieldDefinitionId, ScalarType, SchemaFieldId, Wrapping};
use walker::Walk;

use crate::{
    plan::{DataFieldId, OperationPlanContext},
    response::{GraphqlError, PositionedResponseKey, SafeResponseKey},
};

use super::{ConcreteObjectShapeId, PolymorphicObjectShapeId};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct FieldShapeRecord {
    pub expected_key: SafeResponseKey,
    pub key: PositionedResponseKey,
    pub id: DataFieldId,
    pub required_field_id: Option<SchemaFieldId>,
    pub definition_id: FieldDefinitionId,
    pub shape: Shape,
    pub wrapping: Wrapping,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct FieldShapeId(NonZero<u32>);

impl<'ctx> Walk<OperationPlanContext<'ctx>> for FieldShapeId {
    type Walker<'w> = FieldShape<'w> where 'ctx: 'w;

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

pub(crate) struct FieldShape<'a> {
    pub(super) ctx: OperationPlanContext<'a>,
    pub(super) id: FieldShapeId,
}

impl<'a> FieldShape<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a FieldShapeRecord {
        &self.ctx.operation_solution.shapes[self.id]
    }

    pub(crate) fn errors(&self) -> impl Iterator<Item = &'a GraphqlError> + 'a {
        self.ctx
            .operation_plan
            .query_modifications
            .field_shape_id_to_error_ids
            .find_all(self.id)
            .copied()
            .map(|id| &self.ctx.operation_plan.query_modifications[id])
    }

    pub(crate) fn is_skipped(&self) -> bool {
        self.ctx.operation_plan.query_modifications.skipped_field_shapes[self.id]
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
    ConcreteObject(ConcreteObjectShapeId),
    PolymorphicObject(PolymorphicObjectShapeId),
}

impl Shape {
    pub(crate) fn as_concrete_object(&self) -> Option<ConcreteObjectShapeId> {
        match self {
            Shape::ConcreteObject(id) => Some(*id),
            _ => None,
        }
    }
}
