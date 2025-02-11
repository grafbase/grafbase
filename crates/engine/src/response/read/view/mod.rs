mod de;
mod ser;

use std::{borrow::Cow, sync::Arc};

use crate::{
    prepare::RequiredFieldSet,
    response::{InputObjectId, InputResponseObjectSet, ResponseBuilder, ResponseObject, ResponseValue},
};

// A struct to wrap this ref is overkill, but I've changed this so many times that I'm keeping
// Context as it's easier to modify.
#[derive(Clone, Copy)]
pub(super) struct ViewContext<'a> {
    pub(super) response: &'a ResponseBuilder,
}

#[derive(Clone)]
pub(crate) struct ResponseObjectsView<'a> {
    pub(super) ctx: ViewContext<'a>,
    pub(super) response_object_set: Arc<InputResponseObjectSet>,
    pub(super) selection_set: RequiredFieldSet<'a>,
}

#[derive(Clone)]
pub(crate) struct ResponseObjectsViewWithExtraFields<'a> {
    ctx: ViewContext<'a>,
    response_object_set: Arc<InputResponseObjectSet>,
    selection_set: RequiredFieldSet<'a>,
    extra_constant_fields: Vec<(Cow<'static, str>, serde_json::Value)>,
}

impl<'a> ResponseObjectsView<'a> {
    pub fn iter_with_id(&self) -> impl Iterator<Item = (InputObjectId, ResponseObjectView<'a>)> + '_ {
        self.response_object_set.iter_with_id().map(|(id, obj_ref)| {
            (
                id,
                ResponseObjectView {
                    ctx: self.ctx,
                    response_object: &self.ctx.response.data_parts[obj_ref.id],
                    selection_set: self.selection_set,
                },
            )
        })
    }

    pub fn into_input_object_refs(self) -> Arc<InputResponseObjectSet> {
        self.response_object_set
    }

    pub fn iter(&self) -> impl Iterator<Item = ResponseObjectView<'a>> + '_ {
        self.iter_with_id().map(|(_, view)| view)
    }

    pub fn with_extra_constant_fields(
        self,
        extra_constant_fields: Vec<(Cow<'static, str>, serde_json::Value)>,
    ) -> ResponseObjectsViewWithExtraFields<'a> {
        ResponseObjectsViewWithExtraFields {
            ctx: self.ctx,
            response_object_set: self.response_object_set,
            selection_set: self.selection_set,
            extra_constant_fields,
        }
    }
}

impl ResponseObjectsViewWithExtraFields<'_> {
    pub fn len(&self) -> usize {
        self.response_object_set.len()
    }

    pub fn iter_with_id(&self) -> impl Iterator<Item = (InputObjectId, ResponseObjectViewWithExtraFields<'_>)> + '_ {
        self.response_object_set.iter_with_id().map(move |(id, obj_ref)| {
            (
                id,
                ResponseObjectViewWithExtraFields {
                    ctx: self.ctx,
                    response_object: &self.ctx.response.data_parts[obj_ref.id],
                    selection_set: self.selection_set,
                    extra_constant_fields: &self.extra_constant_fields,
                },
            )
        })
    }
}

pub(crate) struct ResponseObjectView<'a> {
    ctx: ViewContext<'a>,
    response_object: &'a ResponseObject,
    selection_set: RequiredFieldSet<'a>,
}

pub(crate) struct ResponseObjectViewWithExtraFields<'a> {
    ctx: ViewContext<'a>,
    response_object: &'a ResponseObject,
    selection_set: RequiredFieldSet<'a>,
    extra_constant_fields: &'a [(Cow<'static, str>, serde_json::Value)],
}

struct ResponseValueView<'a> {
    ctx: ViewContext<'a>,
    value: &'a ResponseValue,
    selection_set: RequiredFieldSet<'a>,
}
