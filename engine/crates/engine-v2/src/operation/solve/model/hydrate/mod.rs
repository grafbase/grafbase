mod arguments;

pub(crate) use arguments::*;
use id_newtypes::IdRange;
use schema::Schema;

use crate::operation::{InputValueContext, Variables};

use super::{FieldArgumentId, SolvedOperation, SolvedOperationContext};

#[derive(Clone, Copy)]
pub(crate) struct HydratedOperationContext<'a> {
    pub schema: &'a Schema,
    pub operation: &'a SolvedOperation,
    pub variables: &'a Variables,
}

impl<'ctx> From<HydratedOperationContext<'ctx>> for SolvedOperationContext<'ctx> {
    fn from(ctx: HydratedOperationContext<'ctx>) -> Self {
        SolvedOperationContext {
            schema: ctx.schema,
            operation: ctx.operation,
        }
    }
}

impl<'ctx> From<HydratedOperationContext<'ctx>> for InputValueContext<'ctx> {
    fn from(ctx: HydratedOperationContext<'ctx>) -> Self {
        InputValueContext {
            schema: ctx.schema,
            query_input_values: &ctx.operation.query_input_values,
            variables: ctx.variables,
        }
    }
}

impl<'a> SolvedOperationContext<'a> {
    pub fn hydrate_arguments<'w, 'v>(
        &self,
        argument_ids: IdRange<FieldArgumentId>,
        variables: &'v Variables,
    ) -> HydratedFieldArguments<'w>
    where
        'v: 'w,
        'a: 'w,
    {
        HydratedFieldArguments {
            ctx: HydratedOperationContext {
                schema: self.schema,
                operation: self.operation,
                variables,
            },
            ids: argument_ids,
        }
    }
}
