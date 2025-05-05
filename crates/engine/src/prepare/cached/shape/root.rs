use id_newtypes::IdRange;
use operation::{Location, PositionedResponseKey, ResponseKey};
use schema::StringId;
use walker::Walk;

use crate::prepare::{ConcreteShapeId, OperationPlanContext};

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
pub struct DefaultFieldShape {
    pub key: PositionedResponseKey,
    pub value: Option<StringId>,
}
