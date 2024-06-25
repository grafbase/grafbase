use std::sync::Arc;

use super::ResponseBuilder;
mod selection_set;
mod ser;
mod view;

use schema::SchemaWalker;
pub use selection_set::{ReadField, ReadSelectionSet};
pub use view::{ResponseObjectRef, ResponseObjectsView};

impl ResponseBuilder {
    pub fn read<'a>(
        &'a self,
        schema: SchemaWalker<'a, ()>,
        refs: Arc<Vec<ResponseObjectRef>>,
        selection_set: &'a ReadSelectionSet,
    ) -> ResponseObjectsView<'a> {
        ResponseObjectsView {
            schema,
            response: self,
            refs,
            selection_set,
            extra_constant_fields: vec![],
        }
    }
}
