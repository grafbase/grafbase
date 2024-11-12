use walker::Iter;

use crate::{operation::Variables, response::FieldShapeId};

use super::{DataField, HydratedFieldArguments, HydratedOperationContext};

impl<'a> DataField<'a> {
    pub(crate) fn shapes(&self) -> impl Iter<Item = FieldShapeId> + 'a {
        self.ctx.operation_solution[self.as_ref().shape_ids].iter().copied()
    }

    pub(crate) fn response_key_str(&self) -> &'a str {
        &self.ctx.operation_solution.response_keys[self.as_ref().key.response_key]
    }

    pub fn hydrated_arguments<'w, 'v>(&self, variables: impl Into<&'v Variables>) -> HydratedFieldArguments<'w>
    where
        'v: 'w,
        'a: 'w,
    {
        HydratedFieldArguments {
            ctx: HydratedOperationContext {
                schema: self.ctx.schema,
                operation: self.ctx.operation_solution,
                variables: variables.into(),
            },
            ids: self.argument_ids,
        }
    }
}

impl std::fmt::Debug for DataField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataPlanField")
            .field("key", &self.key)
            .field("location", &self.location)
            .field("definition", &self.definition())
            .field("arguments", &self.arguments())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
