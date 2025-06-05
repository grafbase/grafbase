use crate::prepare::RequiredFieldSet;

use super::{ParentObjectSet, ResponseBuilder};
mod ser;
mod view;

pub(crate) use view::*;

impl ResponseBuilder<'_> {
    pub fn read<'a>(&'a self, object_set: ParentObjectSet, requirements: RequiredFieldSet<'a>) -> ParentObjects<'a> {
        ParentObjects {
            ctx: ViewContext { response: self },
            object_set,
            requirements,
        }
    }
}
