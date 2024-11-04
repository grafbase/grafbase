use schema::FieldDefinition;
use walker::{Iter, Walk};

use crate::{
    operation::Location,
    plan::{DataPlanFieldId, DataPlanFieldRecord, FieldArgument, PlanContext, QueryContext, SelectionSet},
    response::PositionedResponseKey,
};

#[derive(Clone, Copy)]
pub(crate) struct DataField<'a> {
    pub(in crate::plan::execution::model) ctx: QueryContext<'a>,
    pub(in crate::plan::execution::model) id: DataPlanFieldId,
}

#[allow(unused)]
impl<'a> DataField<'a> {
    #[allow(clippy::should_implement_trait)]
    fn as_ref(&self) -> &'a DataPlanFieldRecord {
        &self.ctx.operation_plan[self.id]
    }
    pub(crate) fn id(&self) -> DataPlanFieldId {
        self.id
    }
    pub(crate) fn key(&self) -> PositionedResponseKey {
        self.as_ref().key
    }
    pub(crate) fn location(&self) -> Location {
        self.as_ref().location
    }
    pub(crate) fn definition(&self) -> FieldDefinition<'a> {
        self.as_ref().definition_id.walk(self.ctx.schema)
    }
    pub(crate) fn arguments(&self) -> impl Iter<Item = FieldArgument<'a>> + 'a {
        self.as_ref().argument_ids.walk(PlanContext {
            schema: self.ctx.schema,
            operation_plan: self.ctx.operation_plan,
        })
    }
    pub(crate) fn selection_set(&self) -> SelectionSet<'a> {
        let field = self.as_ref();
        SelectionSet {
            ctx: self.ctx,
            item: field.selection_set_record,
            requires_typename: field.selection_set_requires_typename,
        }
    }
}

impl std::fmt::Debug for DataField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataField")
            .field("key", &self.key())
            .field("location", &self.location())
            .field("definition", &self.definition())
            .field("arguments", &self.arguments())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
