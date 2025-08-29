mod ser;

use std::borrow::Cow;

use operation::ResponseKeys;
use schema::{FieldSetRecord, Schema, ValueInjection};

use crate::{
    prepare::RequiredFieldSet,
    response::{ParentObjectId, ParentObjectSet, ResponseBuilder, ResponseObjectId, ResponseObjectRef, ResponseValue},
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

    fn response_keys(&self) -> &'a ResponseKeys {
        &self.response.operation.cached.operation.response_keys
    }
}

pub(crate) struct ParentObjects<'a> {
    pub(super) ctx: ViewContext<'a>,
    pub(super) object_set: ParentObjectSet,
    pub(super) requirements: RequiredFieldSet<'a>,
}

impl<'a> ParentObjects<'a> {
    pub fn len(&self) -> usize {
        self.object_set.len()
    }

    pub fn get_object_ref(&self, id: ParentObjectId) -> Option<&ResponseObjectRef> {
        self.object_set.get(usize::from(id))
    }

    pub fn into_object_set(self) -> ParentObjectSet {
        self.object_set
    }

    pub fn with_extra_constant_fields<'s, 'b, 'view>(
        &'s self,
        extra_constant_fields: &'b [(Cow<'static, str>, serde_json::Value)],
    ) -> ParentObjectsView<'view, WithExtraFields<'view>>
    where
        'b: 'view,
        'a: 'view,
        's: 'view,
    {
        ParentObjectsView {
            ctx: self.ctx,
            object_set: &self.object_set,
            view: WithExtraFields {
                requirements: self.requirements,
                extra_constant_fields,
            },
        }
    }

    pub fn for_injection<'s, 'view>(
        &'s self,
        injection: ValueInjection,
    ) -> ParentObjectsView<'view, ForInjection<'view>>
    where
        'a: 'view,
        's: 'view,
    {
        ParentObjectsView {
            ctx: self.ctx,
            object_set: &self.object_set,
            view: ForInjection {
                requirements: self.requirements,
                injection,
            },
        }
    }

    pub fn iter_with_id(&self) -> impl Iterator<Item = (ParentObjectId, ResponseObjectView<'a>)> + '_ {
        self.object_set.iter_with_id().map(|(id, obj_ref)| {
            (
                id,
                ResponseObjectView {
                    ctx: self.ctx,
                    response_object_id: obj_ref.id,
                    view: self.requirements,
                },
            )
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = ResponseObjectView<'a, RequiredFieldSet<'a>>> + '_ {
        self.object_set.iter().map(|obj_ref| ResponseObjectView {
            ctx: self.ctx,
            response_object_id: obj_ref.id,
            view: self.requirements,
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ParentObjectsView<'a, View = RequiredFieldSet<'a>> {
    pub(super) ctx: ViewContext<'a>,
    pub(super) object_set: &'a ParentObjectSet,
    pub(super) view: View,
}

impl<'a, View: Copy> ParentObjectsView<'a, View> {
    pub fn iter_with_id(&self) -> impl Iterator<Item = (ParentObjectId, ResponseObjectView<'a, View>)> + '_ {
        self.object_set.iter_with_id().map(|(id, obj_ref)| {
            (
                id,
                ResponseObjectView {
                    ctx: self.ctx,
                    response_object_id: obj_ref.id,
                    view: self.view,
                },
            )
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = ResponseObjectView<'a, View>> + '_ {
        self.object_set.iter().map(|obj_ref| ResponseObjectView {
            ctx: self.ctx,
            response_object_id: obj_ref.id,
            view: self.view,
        })
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
            response_object_id: self.response_object_id,
            view: ForFieldSet {
                requirements: self.view,
                field_set,
            },
        }
    }

    pub fn for_injection(self, injection: ValueInjection) -> ResponseObjectView<'a, ForInjection<'a>> {
        ResponseObjectView {
            ctx: self.ctx,
            response_object_id: self.response_object_id,
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
    response_object_id: ResponseObjectId,
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
