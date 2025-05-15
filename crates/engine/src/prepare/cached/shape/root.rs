use id_newtypes::IdRange;
use operation::{Location, PositionedResponseKey, QueryPosition, ResponseKey};
use schema::StringId;
use walker::Walk;

use crate::prepare::{ConcreteShapeId, OperationPlanContext, PartitionFieldId};

use super::ConcreteShape;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct RootFieldsShapeRecord {
    pub concrete_shape_id: ConcreteShapeId,
    pub on_error: OnRootFieldsError,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum OnRootFieldsError {
    PropagateNull {
        error_location_and_key: (Location, ResponseKey),
    },
    Default {
        fields_sorted_by_key: IdRange<DefaultFieldShapeId>,
        error_location_and_key: (Location, ResponseKey),
    },
    Skip,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct RootFieldsShapeId(u16);

impl<'ctx> Walk<OperationPlanContext<'ctx>> for RootFieldsShapeId {
    type Walker<'w>
        = RootFieldsShape<'w>
    where
        'ctx: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationPlanContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        RootFieldsShape {
            ctx: ctx.into(),
            id: self,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RootFieldsShape<'a> {
    pub(super) ctx: OperationPlanContext<'a>,
    pub id: RootFieldsShapeId,
}

impl std::ops::Deref for RootFieldsShape<'_> {
    type Target = RootFieldsShapeRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> RootFieldsShape<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a RootFieldsShapeRecord {
        &self.ctx.cached.shapes[self.id]
    }
    pub(crate) fn concrete_shape(&self) -> ConcreteShape<'a> {
        self.as_ref().concrete_shape_id.walk(self.ctx)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct DefaultFieldShapeId(u32);

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct DefaultFieldShapeRecord {
    pub query_position_before_modifications: QueryPosition,
    pub response_key: ResponseKey,
    pub id: PartitionFieldId,
    pub value: Option<StringId>,
}

#[derive(Clone, Copy)]
pub(crate) struct DefaultFieldShape<'a> {
    pub(super) ctx: OperationPlanContext<'a>,
    pub id: DefaultFieldShapeId,
}

impl<'ctx> Walk<OperationPlanContext<'ctx>> for DefaultFieldShapeId {
    type Walker<'w>
        = DefaultFieldShape<'w>
    where
        'ctx: 'w;

    fn walk<'w>(self, ctx: impl Into<OperationPlanContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        DefaultFieldShape {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::ops::Deref for DefaultFieldShape<'_> {
    type Target = DefaultFieldShapeRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> DefaultFieldShape<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a DefaultFieldShapeRecord {
        &self.ctx.cached.shapes[self.id]
    }

    pub fn key(&self) -> PositionedResponseKey {
        let shape = self.as_ref();
        PositionedResponseKey {
            query_position: Some(shape.query_position_before_modifications),
            response_key: shape.response_key,
        }
        .with_query_position_if(match shape.id {
            PartitionFieldId::Data(id) => self.ctx.plan.query_modifications.included_response_data_fields[id],
            PartitionFieldId::Lookup(_) => false,
            PartitionFieldId::Typename(id) => self.ctx.plan.query_modifications.included_response_typename_fields[id],
        })
    }
}
