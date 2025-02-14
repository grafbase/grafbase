mod ser;

use std::{borrow::Cow, sync::Arc};

use schema::FieldSetRecord;

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
pub(crate) struct ResponseObjectsView<'a, View = RequiredFieldSet<'a>> {
    pub(super) ctx: ViewContext<'a>,
    pub(super) response_object_set: Arc<InputResponseObjectSet>,
    pub(super) view: View,
}

impl<'a, View: Copy> ResponseObjectsView<'a, View> {
    pub fn len(&self) -> usize {
        self.response_object_set.len()
    }

    pub fn iter_with_id(&self) -> impl Iterator<Item = (InputObjectId, ResponseObjectView<'a, View>)> + '_ {
        self.response_object_set.iter_with_id().map(|(id, obj_ref)| {
            (
                id,
                ResponseObjectView {
                    ctx: self.ctx,
                    response_object: &self.ctx.response.data_parts[obj_ref.id],
                    view: self.view,
                },
            )
        })
    }

    pub fn into_input_object_refs(self) -> Arc<InputResponseObjectSet> {
        self.response_object_set
    }

    pub fn iter(&self) -> impl Iterator<Item = ResponseObjectView<'a, View>> + '_ {
        self.iter_with_id().map(|(_, view)| view)
    }
}

impl<'a> ResponseObjectsView<'a, RequiredFieldSet<'a>> {
    pub fn with_extra_constant_fields<'b, 'c>(
        self,
        extra_constant_fields: &'b [(Cow<'static, str>, serde_json::Value)],
    ) -> ResponseObjectsView<'c, WithExtraFields<'c>>
    where
        'b: 'c,
        'a: 'c,
    {
        ResponseObjectsView {
            ctx: self.ctx,
            response_object_set: self.response_object_set,
            view: WithExtraFields {
                requirements: self.view,
                extra_constant_fields,
            },
        }
    }
}

impl<'a> ResponseObjectView<'a, RequiredFieldSet<'a>> {
    pub fn for_field_set<'b, 'c>(self, field_set: &'b FieldSetRecord) -> ResponseObjectView<'c, ForFieldSet<'c>>
    where
        'b: 'c,
        'a: 'c,
    {
        ResponseObjectView {
            ctx: self.ctx,
            response_object: self.response_object,
            view: ForFieldSet {
                requirements: self.view,
                field_set,
            },
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ResponseObjectView<'a, View = RequiredFieldSet<'a>> {
    ctx: ViewContext<'a>,
    response_object: &'a ResponseObject,
    view: View,
}

#[derive(Clone, Copy)]
pub(crate) struct ForFieldSet<'a> {
    requirements: RequiredFieldSet<'a>,
    field_set: &'a FieldSetRecord,
}

#[derive(Clone, Copy)]
pub(crate) struct WithExtraFields<'a> {
    requirements: RequiredFieldSet<'a>,
    extra_constant_fields: &'a [(Cow<'static, str>, serde_json::Value)],
}

struct ResponseValueView<'a, View> {
    ctx: ViewContext<'a>,
    value: &'a ResponseValue,
    view: View,
}
