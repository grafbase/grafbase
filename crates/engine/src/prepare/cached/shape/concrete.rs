use id_newtypes::IdRange;
use schema::{InterfaceDefinitionId, ObjectDefinitionId, UnionDefinitionId};
use walker::{Iter, Walk};

use crate::prepare::{OperationPlanContext, ResponseObjectSetId};

use super::{FieldShape, FieldShapeId, TypenameShapeId};

/// Being concrete does not mean it's only associated with a single object definition id
/// only that we know exactly which fields must be present.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ConcreteShapeRecord {
    pub set_id: Option<ResponseObjectSetId>,
    pub identifier: ObjectIdentifier,
    pub typename_shape_ids: IdRange<TypenameShapeId>,
    // Ordered by (non-derived first / derived last).then(PartitionDataFieldId) which should more or less match the ordering of fields
    // coming in from resolvers.
    pub field_shape_ids: IdRange<FieldShapeId>,
    // If there isn't any derived fields, this will be equal to field_shape_ids.end
    pub derived_field_shape_ids_start: FieldShapeId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct ConcreteShapeId(u32);

impl std::ops::Deref for ConcreteShape<'_> {
    type Target = ConcreteShapeRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'ctx> Walk<OperationPlanContext<'ctx>> for ConcreteShapeId {
    type Walker<'w>
        = ConcreteShape<'w>
    where
        'ctx: 'w;

    fn walk<'w>(self, ctx: impl Into<OperationPlanContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        ConcreteShape {
            ctx: ctx.into(),
            id: self,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ConcreteShape<'a> {
    pub(super) ctx: OperationPlanContext<'a>,
    pub id: ConcreteShapeId,
}

impl<'a> ConcreteShape<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a ConcreteShapeRecord {
        &self.ctx.cached.shapes[self.id]
    }
    pub fn has_errors(&self) -> bool {
        self.ctx.plan.query_modifications.concrete_shape_has_error[self.id]
    }
    pub fn fields(&self) -> impl Iter<Item = FieldShape<'a>> + 'a {
        self.field_shape_ids.walk(self.ctx)
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(crate) enum ObjectIdentifier {
    Known(ObjectDefinitionId),
    UnionTypename(UnionDefinitionId),
    InterfaceTypename(InterfaceDefinitionId),
    Anonymous,
}
