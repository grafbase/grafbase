use schema::FieldDefinition;
use walker::{Iter, Walk};

use crate::{
    operation::Location,
    plan::{
        DataFieldId, DataFieldRecord, FieldArgument, OperationPlanContext, OperationSolutionContext, PlanSelectionSet,
    },
    response::PositionedResponseKey,
};

#[derive(Clone, Copy)]
pub(crate) struct PlanDataField<'a> {
    pub(in crate::plan::execution::model) ctx: OperationPlanContext<'a>,
    pub(in crate::plan::execution::model) id: DataFieldId,
}

#[allow(unused)]
impl<'a> PlanDataField<'a> {
    #[allow(clippy::should_implement_trait)]
    fn as_ref(&self) -> &'a DataFieldRecord {
        &self.ctx.operation_solution[self.id]
    }
    pub(crate) fn id(&self) -> DataFieldId {
        self.id
    }
    pub(crate) fn key(&self) -> PositionedResponseKey {
        self.as_ref().key
    }
    pub(crate) fn response_key_str(&self) -> &'a str {
        &self.ctx.operation_solution.response_keys[self.as_ref().key.response_key]
    }
    pub(crate) fn location(&self) -> Location {
        self.as_ref().location
    }
    pub(crate) fn definition(&self) -> FieldDefinition<'a> {
        self.as_ref().definition_id.walk(self.ctx.schema)
    }
    pub(crate) fn arguments(&self) -> impl Iter<Item = FieldArgument<'a>> + 'a {
        self.as_ref().argument_ids.walk(OperationSolutionContext {
            schema: self.ctx.schema,
            operation_solution: self.ctx.operation_solution,
        })
    }
    pub(crate) fn selection_set(&self) -> PlanSelectionSet<'a> {
        let field = self.as_ref();
        PlanSelectionSet {
            ctx: self.ctx,
            item: field.selection_set_record,
            requires_typename: field.selection_set_requires_typename,
        }
    }
}

impl std::fmt::Debug for PlanDataField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanDataField")
            .field("key", &self.key())
            .field("location", &self.location())
            .field("definition", &self.definition())
            .field("arguments", &self.arguments())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
