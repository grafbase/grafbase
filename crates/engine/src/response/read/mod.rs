use std::sync::Arc;

use crate::operation::RequiredFieldSet;

use super::{InputResponseObjectSet, ResponseBuilder};
mod ser;
mod view;

use schema::Schema;
pub(crate) use view::*;

impl ResponseBuilder {
    #[allow(unused)]
    pub fn read<'a>(
        &'a self,
        schema: &'a Schema,
        response_object_set: Arc<InputResponseObjectSet>,
        selection_set: RequiredFieldSet<'a>,
    ) -> ResponseObjectsView<'a> {
        ResponseObjectsView {
            ctx: ViewContext { response: self },
            response_object_set,
            selection_set,
        }
    }
}
