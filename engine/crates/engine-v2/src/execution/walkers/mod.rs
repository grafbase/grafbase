use schema::{Names, SchemaWalker};

mod collect;
mod field;
mod field_argument;
mod fragment_spread;
mod inline_fragment;
mod selection_set;
mod variables;

pub use collect::*;
pub use field::*;
pub use field_argument::*;
pub use fragment_spread::*;
pub use inline_fragment::*;
pub use selection_set::*;
pub use variables::*;

use super::Variables;
use crate::plan::{OperationPlan, PlanId};

// Not really sure whether walker should keep a reference to this context
// or copy it all the time. Chose the latter for now. ¯\_(ツ)_/¯
#[derive(Clone, Copy)]
pub struct WalkerContext<'a, T> {
    schema_walker: SchemaWalker<'a, T>,
    plan: &'a OperationPlan,
    plan_id: PlanId,
    variables: &'a Variables<'a>,
}

impl<'a, T: Copy> WalkerContext<'a, T> {
    fn walk<U: Copy>(&self, id: U) -> WalkerContext<'a, U> {
        WalkerContext {
            schema_walker: self.schema_walker.walk(id),
            plan: self.plan,
            plan_id: self.plan_id,
            variables: self.variables,
        }
    }
}

impl<'ctx> super::ExecutionContext<'ctx> {
    /// If you do no need to rename anything, use this walker with the schema names.
    pub fn default_walk_selection_set(&self) -> SelectionSetWalker<'ctx> {
        self.walk_selection_set(self.engine.schema.as_ref())
    }

    pub fn walk_selection_set<'a>(&self, names: &'a dyn Names) -> SelectionSetWalker<'a>
    where
        'ctx: 'a,
    {
        let ctx = WalkerContext {
            schema_walker: self.engine.schema.walker(names),
            plan: self.plan,
            plan_id: self.plan_id,
            variables: self.variables,
        };
        SelectionSetWalker {
            ctx,
            id: self.plan.execution_plans[self.plan_id].root.id,
        }
    }

    pub fn default_walk_variables(&self) -> VariablesWalker<'ctx> {
        self.walk_variables(self.engine.schema.as_ref())
    }

    pub fn walk_variables<'a>(&self, names: &'a dyn Names) -> VariablesWalker<'a>
    where
        'ctx: 'a,
    {
        let ctx = WalkerContext {
            schema_walker: self.engine.schema.walker(names),
            plan: self.plan,
            plan_id: self.plan_id,
            variables: self.variables,
        };
        VariablesWalker {
            ctx,
            inner: self.variables,
        }
    }
}
