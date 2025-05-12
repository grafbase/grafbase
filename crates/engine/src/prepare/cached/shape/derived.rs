use id_newtypes::IdRange;
use schema::ObjectDefinitionId;
use walker::{Iter, Walk};

use crate::prepare::{OperationPlanContext, ResponseObjectSetDefinitionId};

use super::{FieldShape, FieldShapeId, TypenameShapeId, TypenameShapeRecord};

/// Being concrete does not mean it's only associated with a single object definition id
/// only that we know exactly which fields must be present.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct DerivedEntityShapeRecord {
    pub set_id: Option<ResponseObjectSetDefinitionId>,
    pub object_definition_id: Option<ObjectDefinitionId>,
    pub typename_shape_ids: IdRange<TypenameShapeId>,
    pub field_shape_ids: IdRange<FieldShapeId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct DerivedEntityShapeId(u32);

impl std::ops::Deref for DerivedEntityShape<'_> {
    type Target = DerivedEntityShapeRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'ctx> Walk<OperationPlanContext<'ctx>> for DerivedEntityShapeId {
    type Walker<'w>
        = DerivedEntityShape<'w>
    where
        'ctx: 'w;

    fn walk<'w>(self, ctx: impl Into<OperationPlanContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        DerivedEntityShape {
            ctx: ctx.into(),
            id: self,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct DerivedEntityShape<'a> {
    pub(super) ctx: OperationPlanContext<'a>,
    pub id: DerivedEntityShapeId,
}

impl<'a> DerivedEntityShape<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a DerivedEntityShapeRecord {
        &self.ctx.cached.shapes[self.id]
    }
    pub(crate) fn fields(&self) -> impl Iter<Item = FieldShape<'a>> + 'a {
        self.as_ref().field_shape_ids.walk(self.ctx)
    }
    pub(crate) fn typename_shapes_slice(&self) -> &'a [TypenameShapeRecord] {
        &self.ctx.cached.shapes[self.as_ref().typename_shape_ids]
    }
}
