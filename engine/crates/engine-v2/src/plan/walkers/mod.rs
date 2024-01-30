use schema::SchemaWalker;

use crate::{
    execution::Variables,
    request::{Operation, OperationWalker, VariableDefinitionWalker},
    response::{ResponseKeys, ResponsePart, ResponsePath, SeedContext},
};

use super::{ConcreteSelectionSetId, OperationPlan, PlanId, PlanInput, PlanOutput};

mod argument;
mod collected;
mod field;
mod fragment_spread;
mod inline_fragment;
mod selection_set;

pub use argument::*;
pub use collected::*;
pub use field::*;
pub use fragment_spread::*;
pub use inline_fragment::*;
pub use selection_set::*;

#[derive(Clone, Copy)]
pub(crate) struct PlanWalker<'a, Item = (), SchemaItem = ()> {
    pub(super) schema_walker: SchemaWalker<'a, SchemaItem>,
    pub(super) operation: &'a OperationPlan,
    pub(super) variables: Option<&'a Variables>,
    pub(super) plan_id: PlanId,
    pub(super) item: Item,
}

impl<'a> std::fmt::Debug for PlanWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanWalker").finish_non_exhaustive()
    }
}

impl<'a, I: Copy, SI> PlanWalker<'a, I, SI>
where
    Operation: std::ops::Index<I>,
{
    pub fn as_ref(&self) -> &'a <Operation as std::ops::Index<I>>::Output {
        &self.operation.bound_operation[self.item]
    }

    #[allow(dead_code)]
    pub fn id(&self) -> I {
        self.item
    }
}

impl<'a> PlanWalker<'a> {
    pub fn schema(&self) -> SchemaWalker<'a> {
        self.schema_walker
    }

    pub fn response_keys(&self) -> &'a ResponseKeys {
        &self.operation.response_keys
    }

    pub fn operation(&self) -> OperationWalker<'a> {
        self.operation.bound_operation.walker_with(self.schema_walker.walk(()))
    }

    pub fn selection_set(self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::RootFields(self)
    }

    pub fn id(&self) -> PlanId {
        self.plan_id
    }

    pub fn output(&self) -> &'a PlanOutput {
        &self.operation.plan_outputs[usize::from(self.plan_id)]
    }

    pub fn input(&self) -> Option<&'a PlanInput> {
        self.operation.plan_inputs[usize::from(self.plan_id)].as_ref()
    }

    pub fn collected_selection_set(&self) -> PlanWalker<'a, ConcreteSelectionSetId, ()> {
        self.walk(self.output().collected_selection_set_id)
    }

    pub fn variable_definition(&self, name: &str) -> Option<VariableDefinitionWalker<'a>> {
        self.bound_walk_with((), ()).variable_definition(name)
    }

    pub fn new_seed<'out>(self, output: &'out mut ResponsePart) -> SeedContext<'out>
    where
        'a: 'out,
    {
        SeedContext::new(self, output)
    }

    pub fn root_error_path(&self, parent: &ResponsePath) -> ResponsePath {
        let mut fields = self.collected_selection_set().fields();
        if fields.len() == 1 {
            parent.child(fields.next().unwrap().as_bound_field().response_edge())
        } else {
            parent.clone()
        }
    }
}

impl<'a, Id> std::ops::Index<Id> for PlanWalker<'a>
where
    OperationPlan: std::ops::Index<Id>,
{
    type Output = <OperationPlan as std::ops::Index<Id>>::Output;
    fn index(&self, index: Id) -> &Self::Output {
        &self.operation[index]
    }
}

impl<'a, I, SI> PlanWalker<'a, I, SI> {
    pub fn walk<I2>(&self, item: I2) -> PlanWalker<'a, I2, SI>
    where
        SI: Copy,
    {
        PlanWalker {
            operation: self.operation,
            variables: self.variables,
            plan_id: self.plan_id,
            schema_walker: self.schema_walker,
            item,
        }
    }

    pub fn walk_with<I2, SI2>(&self, item: I2, schema_item: SI2) -> PlanWalker<'a, I2, SI2> {
        PlanWalker {
            operation: self.operation,
            variables: self.variables,
            plan_id: self.plan_id,
            schema_walker: self.schema_walker.walk(schema_item),
            item,
        }
    }

    pub fn bound_walk_with<I2, SI2: Copy>(&self, item: I2, schema_item: SI2) -> OperationWalker<'a, I2, SI2> {
        self.operation
            .bound_operation
            .walker_with(self.schema_walker.walk(schema_item))
            .walk(item)
    }
}
