use schema::SchemaWalker;

mod field;
mod field_argument;
mod fragment_spread;
mod inline_fragment;
mod selection_set;
mod variables;

pub use field::*;
pub use field_argument::*;
pub use fragment_spread::*;
pub use inline_fragment::*;
pub use selection_set::*;
pub use variables::*;

use super::Variables;
use crate::{
    plan::Attribution,
    request::{BoundSelectionSetId, Operation},
};

// Not really sure whether walker should keep a reference to this context
// or copy it all the time. Chose the latter for now. ¯\_(ツ)_/¯
#[derive(Clone, Copy)]
pub struct WalkerContext<'a, T> {
    pub(super) schema_walker: SchemaWalker<'a, T>,
    pub(super) operation: &'a Operation,
    pub(super) attribution: &'a Attribution,
    pub(super) variables: &'a Variables<'a>,
}

impl<'a, T: Copy> WalkerContext<'a, T> {
    fn walk<U: Copy>(&self, id: U) -> WalkerContext<'a, U> {
        WalkerContext {
            schema_walker: self.schema_walker.walk(id),
            operation: self.operation,
            attribution: self.attribution,
            variables: self.variables,
        }
    }
}

impl<'a> WalkerContext<'a, ()> {
    pub(super) fn walk_selection_set(
        self,
        merged_selection_set_ids: Vec<BoundSelectionSetId>,
    ) -> SelectionSetWalker<'a> {
        SelectionSetWalker {
            ctx: self,
            merged_selection_set_ids,
        }
    }
}
