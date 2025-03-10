use std::num::NonZero;

use id_newtypes::IdRange;
use operation::PositionedResponseKey;
use schema::{InterfaceDefinitionId, ObjectDefinitionId, UnionDefinitionId};
use walker::{Iter, Walk};

use crate::prepare::{OperationPlanContext, ResponseObjectSetDefinitionId};

use super::{FieldShape, FieldShapeId};

/// Being concrete does not mean it's only associated with a single object definition id
/// only that we know exactly which fields must be present.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ConcreteShapeRecord {
    pub set_id: Option<ResponseObjectSetDefinitionId>,
    pub identifier: ObjectIdentifier,
    pub typename_response_keys: Vec<PositionedResponseKey>,
    // Ordered by PartitionDataFieldId which should more or less match the ordering of fields
    // coming in from resolvers.
    pub field_shape_ids: IdRange<FieldShapeId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct ConcreteShapeId(NonZero<u32>);

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
    pub(super) id: ConcreteShapeId,
}

impl<'a> ConcreteShape<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a ConcreteShapeRecord {
        &self.ctx.cached.shapes[self.id]
    }
    pub(crate) fn has_errors(&self) -> bool {
        self.ctx.plan.query_modifications.concrete_shape_has_error[self.id]
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
