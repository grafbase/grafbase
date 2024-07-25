use std::sync::Arc;

use super::{InputdResponseObjectSet, ResponseBuilder};
mod selection_set;
mod ser;
mod view;

use schema::Schema;
pub(crate) use selection_set::*;
pub(crate) use view::*;

impl ResponseBuilder {
    pub fn read<'a>(
        &'a self,
        schema: &'a Schema,
        response_views: &'a ResponseViews,
        response_object_set: Arc<InputdResponseObjectSet>,
        selection_set: ResponseViewSelectionSet,
    ) -> ResponseObjectsView<'a> {
        ResponseObjectsView {
            ctx: ViewContext {
                schema,
                response_views,
                response: self,
            },
            response_object_set,
            selection_set,
        }
    }
}
