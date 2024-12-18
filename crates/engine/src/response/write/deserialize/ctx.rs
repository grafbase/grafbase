use std::cell::{Cell, Ref, RefCell, RefMut};

use crate::{
    operation::{OperationPlanContext, SolvedOperationContext},
    prepare::PreparedOperation,
    response::{ResponseKeys, ResponseValueId, SubgraphResponseRefMut},
};
use itertools::Itertools;
use schema::Schema;

pub(super) struct SeedContext<'ctx> {
    pub schema: &'ctx Schema,
    pub operation: &'ctx PreparedOperation,
    pub subgraph_response: SubgraphResponseRefMut<'ctx>,
    pub bubbling_up_serde_error: Cell<bool>,
    pub path: RefCell<Vec<ResponseValueId>>,
}

impl<'ctx> From<&SeedContext<'ctx>> for SolvedOperationContext<'ctx> {
    fn from(ctx: &SeedContext<'ctx>) -> Self {
        SolvedOperationContext {
            schema: ctx.schema,
            operation: &ctx.operation.cached.solved,
        }
    }
}

impl<'ctx> From<&SeedContext<'ctx>> for OperationPlanContext<'ctx> {
    fn from(ctx: &SeedContext<'ctx>) -> Self {
        OperationPlanContext {
            schema: ctx.schema,
            solved_operation: &ctx.operation.cached.solved,
            operation_plan: &ctx.operation.plan,
        }
    }
}

impl SeedContext<'_> {
    pub(super) fn path(&self) -> RefMut<'_, Vec<ResponseValueId>> {
        self.path.borrow_mut()
    }

    pub(super) fn display_path(&self) -> impl std::fmt::Display + '_ {
        let keys = &self.operation.cached.solved.response_keys;
        let path = self.path.borrow();
        DisplayPath { keys, path }
    }
}

struct DisplayPath<'a> {
    keys: &'a ResponseKeys,
    path: Ref<'a, Vec<ResponseValueId>>,
}

impl std::fmt::Display for DisplayPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            self.path.iter().format_with(".", |value_id, f| match value_id {
                ResponseValueId::Field { key, .. } => f(&format_args!("{}", &self.keys[*key])),
                ResponseValueId::Index { index, .. } => f(&format_args!("{}", index)),
            }),
        ))
    }
}
