mod de;
mod ser;

use std::sync::Arc;

use schema::Schema;

use super::ReadSelectionSet;
use crate::response::{FilteredResponseObjectSet, ResponseBuilder, ResponseObject, ResponseValue};

#[derive(Clone)]
pub(crate) struct ResponseObjectsView<'a> {
    pub(super) schema: &'a Schema,
    pub(super) response: &'a ResponseBuilder,
    pub(super) response_object_set: Arc<FilteredResponseObjectSet>,
    pub(super) selection_set: &'a ReadSelectionSet,
}

#[derive(Clone)]
pub(crate) struct ResponseObjectsViewWithExtraFields<'a> {
    schema: &'a Schema,
    response: &'a ResponseBuilder,
    response_object_set: Arc<FilteredResponseObjectSet>,
    selection_set: &'a ReadSelectionSet,
    extra_constant_fields: Vec<(String, serde_json::Value)>,
}

impl<'a> ResponseObjectsView<'a> {
    pub fn with_extra_constant_fields(
        self,
        extra_constant_fields: Vec<(String, serde_json::Value)>,
    ) -> ResponseObjectsViewWithExtraFields<'a> {
        ResponseObjectsViewWithExtraFields {
            schema: self.schema,
            response: self.response,
            response_object_set: self.response_object_set,
            selection_set: self.selection_set,
            extra_constant_fields,
        }
    }
}

impl<'a> ResponseObjectsViewWithExtraFields<'a> {
    fn iter(&self) -> impl Iterator<Item = ResponseObjectWithExtraFieldsWalker<'_>> + '_ {
        self.response_object_set
            .iter()
            .map(|item| ResponseObjectWithExtraFieldsWalker {
                schema: self.schema,
                response: self.response,
                response_object: &self.response[item.id],
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
            schema: self.view.schema,
            response: self.view.response,
            response_object: &self.view.response[item.id],
            selection_set: self.view.selection_set,
        })
    }
}

pub(crate) struct ResponseObjectView<'a> {
    schema: &'a Schema,
    response: &'a ResponseBuilder,
    response_object: &'a ResponseObject,
    selection_set: &'a ReadSelectionSet,
}

struct ResponseObjectWithExtraFieldsWalker<'a> {
    schema: &'a Schema,
    response: &'a ResponseBuilder,
    response_object: &'a ResponseObject,
    selection_set: &'a ReadSelectionSet,
    extra_constant_fields: &'a [(String, serde_json::Value)],
}

struct ResponseValueWalker<'a> {
    schema: &'a Schema,
    response: &'a ResponseBuilder,
    value: &'a ResponseValue,
    selection_set: &'a ReadSelectionSet,
}
