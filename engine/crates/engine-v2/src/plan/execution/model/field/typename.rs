use schema::CompositeType;
use walker::Walk;

use crate::{
    operation::Location,
    plan::{QueryContext, TypenamePlanField, TypenamePlanFieldId, TypenamePlanFieldRecord},
    response::PositionedResponseKey,
};

#[derive(Clone, Copy)]
pub(crate) struct TypenameField<'a> {
    pub(in crate::plan::execution::model) ctx: QueryContext<'a>,
    pub(in crate::plan::execution::model) id: TypenamePlanFieldId,
}

#[allow(unused)]
impl<'a> TypenameField<'a> {
    #[allow(clippy::should_implement_trait)]
    fn as_ref(&self) -> &'a TypenamePlanFieldRecord {
        &self.ctx.operation_plan[self.id]
    }
    pub(crate) fn key(&self) -> PositionedResponseKey {
        self.as_ref().key
    }
    pub(crate) fn location(&self) -> Location {
        self.as_ref().location
    }
    pub(crate) fn type_condition(&self) -> CompositeType<'a> {
        self.as_ref().type_condition_id.walk(self.ctx.schema)
    }
}

impl std::fmt::Debug for TypenameField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypenameField")
            .field("key", &self.key())
            .field("location", &self.location())
            .field("type_condition", &self.type_condition())
            .finish()
    }
}
