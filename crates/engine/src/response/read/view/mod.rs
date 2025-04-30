mod ser;

use std::borrow::Cow;

use schema::{FieldSetRecord, Schema, ValueInjection};

use crate::{
    prepare::RequiredFieldSet,
    response::{ParentObjectId, ParentObjects, ResponseBuilder, ResponseObject, ResponseValue},
};

// A struct to wrap this ref is overkill, but I've changed this so many times that I'm keeping
// Context as it's easier to modify.
#[derive(Clone, Copy)]
pub(super) struct ViewContext<'a> {
    pub(super) response: &'a ResponseBuilder<'a>,
}

impl<'a> ViewContext<'a> {
    fn schema(&self) -> &'a Schema {
        self.response.schema
    }
}

#[derive(Clone)]
pub(crate) struct ParentObjectsView<'a, View = RequiredFieldSet<'a>> {
    pub(super) ctx: ViewContext<'a>,
    pub(super) parent_objects: ParentObjects,
    pub(super) view: View,
}

impl<'a, View: Copy> ParentObjectsView<'a, View> {
    pub fn len(&self) -> usize {
        self.parent_objects.len()
    }

    pub fn iter_with_id(&self) -> impl Iterator<Item = (ParentObjectId, ResponseObjectView<'a, View>)> + '_ {
        self.parent_objects.iter_with_id().map(|(id, obj_ref)| {
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

    pub fn into_object_set(self) -> ParentObjects {
        self.parent_objects
    }

    pub fn iter(&self) -> impl Iterator<Item = ResponseObjectView<'a, View>> + '_ {
        self.iter_with_id().map(|(_, view)| view)
    }
}

impl<'a> ParentObjectsView<'a, RequiredFieldSet<'a>> {
    pub fn with_extra_constant_fields<'b, 'c>(
        self,
        extra_constant_fields: &'b [(Cow<'static, str>, serde_json::Value)],
    ) -> ParentObjectsView<'c, WithExtraFields<'c>>
    where
        'b: 'c,
        'a: 'c,
    {
        ParentObjectsView {
            ctx: self.ctx,
            parent_objects: self.parent_objects,
            view: WithExtraFields {
                requirements: self.view,
                extra_constant_fields,
            },
        }
    }

    pub fn for_injection(self, injection: ValueInjection) -> ParentObjectsView<'a, ForInjection<'a>> {
        ParentObjectsView {
            ctx: self.ctx,
            parent_objects: self.parent_objects,
            view: ForInjection {
                requirements: self.view,
                injection,
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

    pub fn for_injection(self, injection: ValueInjection) -> ResponseObjectView<'a, ForInjection<'a>> {
        ResponseObjectView {
            ctx: self.ctx,
            response_object: self.response_object,
            view: ForInjection {
                requirements: self.view,
                injection,
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
pub(crate) struct ForInjection<'a> {
    requirements: RequiredFieldSet<'a>,
    injection: ValueInjection,
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
