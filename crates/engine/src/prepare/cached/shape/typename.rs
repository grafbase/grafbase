use operation::{Location, PositionedResponseKey, QueryPosition, ResponseKey};
use walker::Walk;

use crate::prepare::{OperationPlanContext, TypenameFieldId};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct TypenameShapeRecord {
    pub query_position_before_modifications: Option<QueryPosition>,
    pub response_key: ResponseKey,
    pub id: TypenameFieldId,
    pub location: Location,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct TypenameShapeId(u32);

impl<'ctx> Walk<OperationPlanContext<'ctx>> for TypenameShapeId {
    type Walker<'w>
        = TypenameShape<'w>
    where
        'ctx: 'w;

    fn walk<'w>(self, ctx: impl Into<OperationPlanContext<'ctx>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'ctx: 'w,
    {
        TypenameShape {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::ops::Deref for TypenameShape<'_> {
    type Target = TypenameShapeRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct TypenameShape<'a> {
    pub(super) ctx: OperationPlanContext<'a>,
    pub id: TypenameShapeId,
}

impl std::fmt::Debug for TypenameShape<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypenameShape")
            .field(
                "response_key",
                &&self.ctx.cached.operation.response_keys[self.as_ref().response_key],
            )
            .finish()
    }
}

#[allow(unused)]
impl<'a> TypenameShape<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a TypenameShapeRecord {
        &self.ctx.cached.shapes[self.id]
    }

    pub fn key(&self) -> PositionedResponseKey {
        let shape = self.as_ref();
        PositionedResponseKey {
            query_position: shape.query_position_before_modifications,
            response_key: shape.response_key,
        }
        .with_query_position_if(self.ctx.plan.query_modifications.included_response_typename_fields[shape.id])
    }
}
