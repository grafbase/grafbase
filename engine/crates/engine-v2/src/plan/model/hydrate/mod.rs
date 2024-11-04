mod arguments;

pub(crate) use arguments::*;
use id_newtypes::IdRange;

use crate::operation::Variables;

use super::{FieldArgumentId, PlanContext};

impl<'a> PlanContext<'a> {
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
            schema: self.schema,
            operation_plan: self.operation_plan,
            variables,
            ids: argument_ids,
        }
    }
}
