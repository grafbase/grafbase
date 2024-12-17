use walker::Walk;

use super::{DataField, ExecutableDirectiveId, Field, SelectionSet, SelectionSetRecord, TypenameField};

impl<'a> DataField<'a> {
    pub fn key_str(&self) -> &'a str {
        &self.ctx.operation.response_keys[self.key]
    }
}

impl<'a> TypenameField<'a> {
    pub fn key_str(&self) -> &'a str {
        &self.ctx.operation.response_keys[self.key]
    }
}

impl<'a> Field<'a> {
    pub fn selection_set(&self) -> SelectionSet<'a> {
        match self {
            Field::Data(data) => data.selection_set(),
            Field::Typename(field) => SelectionSetRecord::empty().walk(field.ctx),
        }
    }
    pub fn directive_ids(&self) -> &'a [ExecutableDirectiveId] {
        match self {
            Field::Data(data) => data.as_ref().directive_ids.as_slice(),
            Field::Typename(typename) => typename.as_ref().directive_ids.as_slice(),
        }
    }
}
