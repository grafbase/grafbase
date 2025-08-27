use operation::{PositionedResponseKey, QueryPosition, ResponseKey};
use schema::{EnumDefinitionId, ScalarType, Wrapping};
use walker::Walk;

use crate::prepare::{DataOrLookupField, DataOrLookupFieldId, OperationPlanContext, QueryErrorId};

use super::{ConcreteShapeId, DerivedEntityShape, DerivedEntityShapeId, PolymorphicShapeId};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct FieldShapeRecord {
    pub expected_key: ResponseKey,
    pub query_position_before_modifications: Option<QueryPosition>,
    pub response_key: ResponseKey,
    // TODO: merge both discriminant into a u8?
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
    pub id: FieldShapeId,
}

#[allow(unused)]
impl<'a> FieldShape<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a FieldShapeRecord {
        &self.ctx.cached.shapes[self.id]
    }

    pub fn partition_field(&self) -> DataOrLookupField<'a> {
        self.as_ref().id.walk(self.ctx)
    }

    pub fn key(&self) -> PositionedResponseKey {
        let shape = self.as_ref();
        PositionedResponseKey {
            query_position: shape.query_position_before_modifications,
            response_key: shape.response_key,
        }
        .with_query_position_if(match shape.id {
            DataOrLookupFieldId::Data(id) => self.ctx.plan.query_modifications.included_response_data_fields[id],
            _ => false,
        })
    }

    pub fn error_ids(&self) -> impl Iterator<Item = QueryErrorId> + 'a {
        self.ctx
            .plan
            .query_modifications
            .field_shape_id_to_error_ids
            .find_all(self.id)
            .copied()
    }

    pub fn is_absent(&self) -> bool {
        match self.as_ref().id {
            DataOrLookupFieldId::Data(id) => {
                !self.ctx.plan.query_modifications.included_subgraph_request_data_fields[id]
            }
            DataOrLookupFieldId::Lookup(_) => false,
        }
    }

    pub fn is_included(&self) -> bool {
        match self.as_ref().id {
            DataOrLookupFieldId::Data(id) => self.ctx.plan.query_modifications.included_response_data_fields[id],
            _ => false,
        }
    }

    pub fn derive_entity_shape(&self) -> Option<DerivedEntityShape<'a>> {
        match self.shape {
            Shape::DeriveEntity(id) => Some(DerivedEntityShape { ctx: self.ctx, id }),
            _ => None,
        }
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
    DeriveEntity(DerivedEntityShapeId),
    DeriveFrom(Option<QueryPosition>),
    DeriveFromScalar,
}

impl Shape {
    pub fn as_concrete(&self) -> Option<ConcreteShapeId> {
        match self {
            Shape::Concrete(id) => Some(*id),
            _ => None,
        }
    }

    pub fn is_derive_entity(&self) -> bool {
        matches!(self, Shape::DeriveEntity(_))
    }

    pub fn is_derive_from_scalar(&self) -> bool {
        matches!(self, Shape::DeriveFromScalar)
    }

    pub fn as_derive_from_query_position(self) -> Option<QueryPosition> {
        match self {
            Shape::DeriveFrom(pos) => pos,
            _ => None,
        }
    }
}
