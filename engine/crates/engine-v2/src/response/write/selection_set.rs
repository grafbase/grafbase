use core::num::NonZeroUsize;

use crate::{
    execution::ExecStringId,
    formatter::{ContextAwareDebug, FormatterContext, FormatterContextHolder},
    request::OperationFieldId,
};

/// Selection set used to write data into the response.
/// Used for plan outputs.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct WriteSelectionSet {
    dense_capacity: Option<NonZeroUsize>,
    items: Vec<WriteSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteSelection {
    pub operation_field_id: OperationFieldId,
    pub response_position: usize,
    pub response_name: ExecStringId,
    pub subselection: WriteSelectionSet,
}

impl WriteSelectionSet {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn new(dense_capacity: Option<NonZeroUsize>, items: Vec<WriteSelection>) -> Self {
        Self { dense_capacity, items }
    }

    pub fn iter(&self) -> impl Iterator<Item = &WriteSelection> {
        self.items.iter()
    }
}

impl ContextAwareDebug for WriteSelectionSet {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriteSelectionSet")
            .field("items", &ctx.debug(&self.items))
            .finish()
    }
}

impl ContextAwareDebug for WriteSelection {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = ctx.strings[self.response_name].to_string();
        f.debug_struct("WriteSelection")
            .field("name", &name)
            .field("subselection", &ctx.debug(&self.subselection))
            .finish()
    }
}
