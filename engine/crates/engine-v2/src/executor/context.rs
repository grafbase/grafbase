use schema::Names;

use crate::{
    request::{Operation, OperationWalker},
    Engine,
};

/// Data available during the executor life during its build & execution phases.
#[derive(Clone)]
pub struct ExecutorContext<'a> {
    pub engine: &'a Engine,
    pub operation: &'a Operation,
}

impl<'ctx> ExecutorContext<'ctx> {
    /// If you do no need to rename anything, use this walker with the schema names.
    pub fn default_walker(&self) -> OperationWalker<'_> {
        self.walker(&self.engine.schema)
    }

    pub fn walker<'a>(&self, names: &'a dyn Names) -> OperationWalker<'a>
    where
        'ctx: 'a,
    {
        OperationWalker {
            schema: self.engine.schema.walker(names),
            operation: self.operation,
        }
    }
}
