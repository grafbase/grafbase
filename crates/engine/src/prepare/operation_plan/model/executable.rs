use walker::{Iter, Walk};

use super::Executable;

impl<'a> Executable<'a> {
    #[allow(unused)]
    pub(crate) fn parent_count(&self) -> usize {
        match self {
            Executable::Plan(plan) => plan.parent_count,
            Executable::ResponseModifier(modifier) => modifier.parent_count,
        }
    }

    pub(crate) fn children(&self) -> impl Iter<Item = Executable<'a>> + 'a {
        match self {
            Executable::Plan(plan) => plan.as_ref().children_ids.walk(plan.ctx),
            Executable::ResponseModifier(modifier) => modifier.as_ref().children_ids.walk(modifier.ctx),
        }
    }
}
