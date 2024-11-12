use std::sync::Arc;

use super::{InputResponseObjectSet, ResponseBuilder};
mod ser;
mod view;

use schema::{FieldSetRecord, Schema};
pub(crate) use view::*;

impl ResponseBuilder {
    #[allow(unused)]
    pub fn read<'a>(
        &'a self,
        schema: &'a Schema,
        response_object_set: Arc<InputResponseObjectSet>,
        selection_set: &'a FieldSetRecord,
    ) -> ResponseObjectsView<'a> {
        ResponseObjectsView {
            ctx: ViewContext { schema, response: self },
            response_object_set,
            selection_set,
        }
    }
}
