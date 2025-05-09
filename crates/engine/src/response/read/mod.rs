use crate::prepare::RequiredFieldSet;

use super::{ParentObjects, ResponseBuilder};
mod ser;
mod view;

pub(crate) use view::*;

impl ResponseBuilder<'_> {
    pub fn read<'a>(
        &'a self,
        response_object_set: ParentObjects,
        selection_set: RequiredFieldSet<'a>,
    ) -> ParentObjectsView<'a> {
        ParentObjectsView {
            ctx: ViewContext { response: self },
            parent_objects: response_object_set,
            view: selection_set,
        }
    }
}
