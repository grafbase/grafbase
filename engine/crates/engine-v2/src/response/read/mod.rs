use std::sync::Arc;

use super::{InputResponseObjectSet, ResponseBuilder};
mod old_view;
mod selection_set;
mod ser;
mod view;

pub(crate) use old_view::*;
use schema::{RequiredFieldSetRecord, Schema};
pub(crate) use selection_set::*;
pub(crate) use view::*;

impl ResponseBuilder {
    #[allow(unused)]
    pub fn read<'a>(
        &'a self,
        schema: &'a Schema,
        response_object_set: Arc<InputResponseObjectSet>,
        selection_set: &'a RequiredFieldSetRecord,
    ) -> ResponseObjectsView<'a> {
        ResponseObjectsView {
            ctx: ViewContext { schema, response: self },
            response_object_set,
            selection_set,
        }
    }

    pub fn old_read<'a>(
        &'a self,
        schema: &'a Schema,
        response_views: &'a ResponseViews,
        response_object_set: Arc<InputResponseObjectSet>,
        selection_set: ResponseViewSelectionSet,
    ) -> OldResponseObjectsView<'a> {
        OldResponseObjectsView {
            ctx: OldViewContext {
                schema,
                response_views,
                response: self,
            },
            response_object_set,
            selection_set,
        }
    }
}
