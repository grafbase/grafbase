mod de;
mod ser;

use std::sync::Arc;

use schema::Schema;

use super::{ResponseViewSelectionSet, ResponseViews};
use crate::response::{InputResponseObjectSet, ResponseBuilder, ResponseObject, ResponseValue};

#[derive(Clone, Copy)]
pub(super) struct OldViewContext<'a> {
    pub(super) schema: &'a Schema,
    pub(super) response_views: &'a ResponseViews,
    pub(super) response: &'a ResponseBuilder,
}

#[derive(Clone)]
pub(crate) struct OldResponseObjectsView<'a> {
    pub(super) ctx: OldViewContext<'a>,
    pub(super) response_object_set: Arc<InputResponseObjectSet>,
    pub(super) selection_set: ResponseViewSelectionSet,
}

#[derive(Clone)]
pub(crate) struct OldResponseObjectsViewWithExtraFields<'a> {
    ctx: OldViewContext<'a>,
    response_object_set: Arc<InputResponseObjectSet>,
    selection_set: ResponseViewSelectionSet,
    extra_constant_fields: Vec<(String, serde_json::Value)>,
}

impl<'a> OldResponseObjectsView<'a> {
    pub fn with_extra_constant_fields(
        self,
        extra_constant_fields: Vec<(String, serde_json::Value)>,
    ) -> OldResponseObjectsViewWithExtraFields<'a> {
        OldResponseObjectsViewWithExtraFields {
            ctx: self.ctx,
            response_object_set: self.response_object_set,
            selection_set: self.selection_set,
            extra_constant_fields,
        }
    }
}

impl<'a> OldResponseObjectsViewWithExtraFields<'a> {
    pub fn iter(&self) -> impl Iterator<Item = OldResponseObjectWithExtraFieldsWalker<'_>> + '_ {
        self.response_object_set
            .iter()
            .map(|item| OldResponseObjectWithExtraFieldsWalker {
                ctx: self.ctx,

                response_object: &self.ctx.response[item.id],
                selection_set: self.selection_set,
                extra_constant_fields: &self.extra_constant_fields,
            })
    }
}

impl<'a> IntoIterator for OldResponseObjectsView<'a> {
    type Item = OldResponseObjectView<'a>;
    type IntoIter = OldResponseObjectsViewIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        OldResponseObjectsViewIterator { view: self, idx: 0 }
    }
}

pub(crate) struct OldResponseObjectsViewIterator<'a> {
    view: OldResponseObjectsView<'a>,
    idx: usize,
}

impl<'a> Iterator for OldResponseObjectsViewIterator<'a> {
    type Item = OldResponseObjectView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.view.response_object_set.get(self.idx)?;
        self.idx += 1;
        Some(OldResponseObjectView {
            ctx: self.view.ctx,
            response_object: &self.view.ctx.response[item.id],
            selection_set: self.view.selection_set,
        })
    }
}

pub(crate) struct OldResponseObjectView<'a> {
    ctx: OldViewContext<'a>,
    response_object: &'a ResponseObject,
    selection_set: ResponseViewSelectionSet,
}

pub(crate) struct OldResponseObjectWithExtraFieldsWalker<'a> {
    ctx: OldViewContext<'a>,
    response_object: &'a ResponseObject,
    selection_set: ResponseViewSelectionSet,
    extra_constant_fields: &'a [(String, serde_json::Value)],
}

struct ResponseValueWalker<'a> {
    ctx: OldViewContext<'a>,
    value: &'a ResponseValue,
    selection_set: ResponseViewSelectionSet,
}
