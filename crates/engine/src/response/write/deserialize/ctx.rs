use std::cell::{Cell, Ref, RefCell, RefMut};

use crate::{
    prepare::{CachedOperationContext, OperationPlanContext, PreparedOperation},
    response::{ResponseValueId, SharedResponsePartBuilder},
};
use itertools::Itertools;
use operation::ResponseKeys;
use schema::Schema;

pub(super) struct SeedContext<'ctx> {
    pub schema: &'ctx Schema,
    pub operation: &'ctx PreparedOperation,
    pub response: SharedResponsePartBuilder<'ctx>,
    pub bubbling_up_serde_error: Cell<bool>,
    pub path: RefCell<Vec<ResponseValueId>>,
}

impl<'ctx> From<&SeedContext<'ctx>> for CachedOperationContext<'ctx> {
    fn from(ctx: &SeedContext<'ctx>) -> Self {
        CachedOperationContext {
            schema: ctx.schema,
            cached: &ctx.operation.cached,
        }
    }
}

impl<'ctx> From<&SeedContext<'ctx>> for OperationPlanContext<'ctx> {
    fn from(ctx: &SeedContext<'ctx>) -> Self {
        OperationPlanContext {
            schema: ctx.schema,
            cached: &ctx.operation.cached,
            plan: &ctx.operation.plan,
        }
    }
}

impl<'ctx> SeedContext<'ctx> {
    pub(super) fn response_keys(&self) -> &'ctx ResponseKeys {
        &self.operation.cached.operation.response_keys
    }

    pub(super) fn path_mut(&self) -> RefMut<'_, Vec<ResponseValueId>> {
        self.path.borrow_mut()
    }

    pub(super) fn path(&self) -> Ref<'_, Vec<ResponseValueId>> {
        self.path.borrow()
    }

    pub(super) fn display_path(&self) -> impl std::fmt::Display + '_ {
        let keys = &self.operation.cached.operation.response_keys;
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
