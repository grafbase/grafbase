use std::sync::Arc;

use crate::prepare::RequiredFieldSet;

use super::{InputResponseObjectSet, ResponseBuilder};
mod ser;
mod view;

pub(crate) use view::*;

impl ResponseBuilder {
    pub fn read<'a>(
        &'a self,
        response_object_set: Arc<InputResponseObjectSet>,
        selection_set: RequiredFieldSet<'a>,
    ) -> ResponseObjectsView<'a> {
        ResponseObjectsView {
            ctx: ViewContext { response: self },
            response_object_set,
            view: selection_set,
        }
    }
}
