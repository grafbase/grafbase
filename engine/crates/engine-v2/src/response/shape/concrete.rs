use std::num::NonZero;

use id_newtypes::IdRange;
use schema::{InterfaceDefinitionId, ObjectDefinitionId, UnionDefinitionId};
use walker::{Iter, Walk};

use crate::{
    operation::{OperationPlanContext, ResponseObjectSetDefinitionId},
    response::PositionedResponseKey,
};

use super::{FieldShape, FieldShapeId};

/// Being concrete does not mean it's only associated with a single object definition id
/// only that we know exactly which fields must be present for one or multiple of them.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ConcreteObjectShapeRecord {
    pub set_id: Option<ResponseObjectSetDefinitionId>,
    pub identifier: ObjectIdentifier,
    pub typename_response_edges: Vec<PositionedResponseKey>,
    // Sorted by expected_key
    pub field_shape_ids: IdRange<FieldShapeId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct ConcreteObjectShapeId(NonZero<u32>);

impl std::ops::Deref for ConcreteObjectShape<'_> {
    type Target = ConcreteObjectShapeRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'ctx> Walk<OperationPlanContext<'ctx>> for ConcreteObjectShapeId {
    type Walker<'w> = ConcreteObjectShape<'w> where 'ctx: 'w;

    fn walk<'w>(self, ctx: impl Into<OperationPlanContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        ConcreteObjectShape {
            ctx: ctx.into(),
            id: self,
        }
    }
}

pub(crate) struct ConcreteObjectShape<'a> {
    pub(super) ctx: OperationPlanContext<'a>,
    pub(super) id: ConcreteObjectShapeId,
}

impl<'a> ConcreteObjectShape<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a ConcreteObjectShapeRecord {
        &self.ctx.operation_solution.shapes[self.id]
    }
    pub(crate) fn has_errors(&self) -> bool {
        self.ctx.operation_plan.query_modifications.concrete_shape_has_error[self.id]
    }
    pub(crate) fn fields(&self) -> impl Iter<Item = FieldShape<'a>> + 'a {
        self.as_ref().field_shape_ids.walk(self.ctx)
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(crate) enum ObjectIdentifier {
    Known(ObjectDefinitionId),
    UnionTypename(UnionDefinitionId),
    InterfaceTypename(InterfaceDefinitionId),
    Anonymous,
}
