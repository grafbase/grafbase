use engine::RequestHeaders;
use schema::SchemaWalker;

use super::Variables;
use crate::{
    plan::PlanOutput,
    request::{OperationWalker, PlanExt, PlanOperationWalker, VariablesWalker},
    response::{ExecutorOutput, ResponseBoundaryItem, ResponseObjectWriter},
    Engine,
};

/// Data available during the executor life during its build & execution phases.
#[derive(Clone, Copy)]
pub(crate) struct ExecutionContext<'ctx> {
    pub engine: &'ctx Engine,
    pub walker: OperationWalker<'ctx>,
    pub(super) variables: &'ctx Variables<'ctx>,
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
            .with_ext(PlanExt {
                attibution: &output.attribution,
                variables: self.variables,
            })
            .walk(output)
    }

    pub fn writer<'a>(
        &self,
        data_part: &'a mut ExecutorOutput,
        boundary_item: &'a ResponseBoundaryItem,
        output: &'a PlanOutput,
    ) -> ResponseObjectWriter<'a>
    where
        'ctx: 'a,
    {
        ResponseObjectWriter::new(
            self.walker.with_ext(PlanExt {
                attibution: &output.attribution,
                variables: self.variables,
            }),
            data_part,
            boundary_item,
            &output.expectation,
        )
    }

    pub fn header(&self, name: &str) -> Option<&'ctx str> {
        self.request_headers.find(name)
    }
}
