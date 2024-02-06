use engine_value::ConstValue;
use schema::{InputValueId, InputValueWalker};

use crate::request::BoundFieldArgument;

use super::PlanWalker;

pub type PlanInputValue<'a> = PlanWalker<'a, &'a BoundFieldArgument, InputValueId>;

impl<'a> PlanInputValue<'a> {
    // Value in the query, before variable resolution.
    pub fn query_value(&self) -> &engine_value::Value {
        &self.item.value
    }

    pub fn resolved_value(&self) -> ConstValue {
        // TODO: ugly as hell
        let variables = self.variables.unwrap();
        // not really efficient, but works.
        self.item
            .value
            .clone()
            .into_const_with::<()>(|name| {
                Ok(variables
                    .get(&name)
                    .expect("Would have failed at validation")
                    .value
                    .clone()
                    .unwrap_or_default())
            })
            .unwrap()
    }
}

impl<'a> std::ops::Deref for PlanInputValue<'a> {
    type Target = InputValueWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}
