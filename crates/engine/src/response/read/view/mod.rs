mod de;
mod ser;

use std::sync::Arc;

use schema::{FieldSetRecord, Schema};

use crate::response::{InputResponseObjectSet, ResponseBuilder, ResponseObject, ResponseValue};

#[derive(Clone, Copy)]
pub(super) struct ViewContext<'a> {
    pub(super) schema: &'a Schema,
    pub(super) response: &'a ResponseBuilder,
}

#[derive(Clone)]
pub(crate) struct ResponseObjectsView<'a> {
    pub(super) ctx: ViewContext<'a>,
    pub(super) response_object_set: Arc<InputResponseObjectSet>,
    pub(super) selection_set: &'a FieldSetRecord,
}

#[derive(Clone)]
pub(crate) struct ResponseObjectsViewWithExtraFields<'a> {
    ctx: ViewContext<'a>,
    response_object_set: Arc<InputResponseObjectSet>,
    selection_set: &'a FieldSetRecord,
    extra_constant_fields: Vec<(String, serde_json::Value)>,
}

impl<'a> ResponseObjectsView<'a> {
    #[allow(unused)]
    pub fn with_extra_constant_fields(
        self,
        extra_constant_fields: Vec<(String, serde_json::Value)>,
    ) -> ResponseObjectsViewWithExtraFields<'a> {
        ResponseObjectsViewWithExtraFields {
            ctx: self.ctx,
            response_object_set: self.response_object_set,
            selection_set: self.selection_set,
            extra_constant_fields,
        }
    }
}

impl<'a> ResponseObjectsViewWithExtraFields<'a> {
    pub fn iter(&self) -> impl Iterator<Item = ResponseObjectViewWithExtraFields<'_>> + '_ {
        self.response_object_set
            .iter()
            .map(|item| ResponseObjectViewWithExtraFields {
                ctx: self.ctx,

                response_object: &self.ctx.response[item.id],
                selection_set: self.selection_set,
                extra_constant_fields: &self.extra_constant_fields,
            })
    }
}

impl<'a> IntoIterator for ResponseObjectsView<'a> {
    type Item = ResponseObjectView<'a>;
    type IntoIter = ResponseObjectsViewIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ResponseObjectsViewIterator { view: self, idx: 0 }
    }
}

pub(crate) struct ResponseObjectsViewIterator<'a> {
    view: ResponseObjectsView<'a>,
    idx: usize,
}

impl<'a> Iterator for ResponseObjectsViewIterator<'a> {
    type Item = ResponseObjectView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.view.response_object_set.get(self.idx)?;
        self.idx += 1;
        Some(ResponseObjectView {
            ctx: self.view.ctx,
            response_object: &self.view.ctx.response[item.id],
            selection_set: self.view.selection_set,
        })
    }
}

pub(crate) struct ResponseObjectView<'a> {
    ctx: ViewContext<'a>,
    response_object: &'a ResponseObject,
    selection_set: &'a FieldSetRecord,
}

pub(crate) struct ResponseObjectViewWithExtraFields<'a> {
    ctx: ViewContext<'a>,
    response_object: &'a ResponseObject,
    selection_set: &'a FieldSetRecord,
    extra_constant_fields: &'a [(String, serde_json::Value)],
}

struct ResponseValueView<'a> {
    ctx: ViewContext<'a>,
    value: &'a ResponseValue,
    selection_set: &'a FieldSetRecord,
}
