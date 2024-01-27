use engine::RequestHeaders;
use schema::SchemaWalker;

use super::Variables;
use crate::{
    plan::PlanOutput,
    request::{ExecutorWalkContext, OperationWalker, PlanOperationWalker, VariablesWalker},
    response::{ExecutorOutput, SeedContext},
    Engine,
};

/// Data available during the executor life during its build & execution phases.
#[derive(Clone, Copy)]
pub(crate) struct ExecutionContext<'ctx> {
    pub engine: &'ctx Engine,
    pub walker: OperationWalker<'ctx>,
    pub(super) variables: &'ctx Variables,
    pub(super) request_headers: &'ctx RequestHeaders,
}

impl<'ctx> ExecutionContext<'ctx> {
    pub fn schema(&self) -> SchemaWalker<'ctx, ()> {
        self.walker.schema()
    }

    pub fn variables(&self) -> VariablesWalker<'ctx> {
        self.walker.walk(self.variables)
    }

    pub fn walk<'p>(&self, output: &'p PlanOutput) -> PlanOperationWalker<'p>
    where
        'ctx: 'p,
    {
        self.walker
            .with_ctx(ExecutorWalkContext {
                attribution: &output.attribution,
                variables: self.variables,
            })
            .walk(output)
    }

    pub fn seed_ctx<'a>(&self, data_part: &'a mut ExecutorOutput, output: &'a PlanOutput) -> SeedContext<'a>
    where
        'ctx: 'a,
    {
        SeedContext::new(
            self.walker.with_ctx(ExecutorWalkContext {
                attribution: &output.attribution,
                variables: self.variables,
            }),
            data_part,
            output,
        )
    }

    pub fn header(&self, name: &str) -> Option<&'ctx str> {
        self.request_headers.find(name)
    }
}
