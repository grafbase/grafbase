use schema::ObjectId;

use super::OperationFieldId;
use crate::{
    execution::StrId,
    formatter::{ContextAwareDebug, FormatterContext, FormatterContextHolder},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OperationPath(im::Vector<OperationPathSegment>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationPathSegment {
    // Keeping the actual id around for debug/print/...
    pub operation_field_id: OperationFieldId,
    // Actual needed fields.
    pub type_condition: Option<ResolvedTypeCondition>,
    pub position: usize,
    pub name: StrId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTypeCondition(Vec<ObjectId>);

impl ResolvedTypeCondition {
    pub fn new(object_ids: Vec<ObjectId>) -> Self {
        Self(object_ids)
    }

    pub fn matches(&self, object_id: ObjectId) -> bool {
        self.0.contains(&object_id)
    }
}

impl<'a> IntoIterator for &'a OperationPath {
    type Item = &'a OperationPathSegment;

    type IntoIter = <&'a im::Vector<OperationPathSegment> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl OperationPath {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn child(&self, segment: OperationPathSegment) -> Self {
        let mut child = self.clone();
        child.0.push_back(segment);
        child
    }
}

impl ContextAwareDebug for OperationPath {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|segment| ctx.debug(segment)))
            .finish()
    }
}

impl ContextAwareDebug for OperationPathSegment {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_condition = self.type_condition.as_ref().map(|cond| {
            cond.0
                .clone()
                .into_iter()
                .map(|object_id| ctx.schema[ctx.schema[object_id].name].to_string())
        });
        f.debug_struct("ResponsePathSegment")
            .field("name", &ctx.strings[self.name].to_string())
            .field("type_condition", &type_condition)
            .finish()
    }
}
