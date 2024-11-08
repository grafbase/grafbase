use walker::{Iter, Walk};

use crate::plan::PlanSelectionSetRecord;

use super::{DataField, Field, QueryContext, TypenameField};

#[derive(Clone, Copy)]
pub(crate) struct SelectionSet<'a> {
    pub(in crate::plan::execution::model) ctx: QueryContext<'a>,
    pub(in crate::plan::execution::model) item: PlanSelectionSetRecord,
    pub(in crate::plan::execution::model) requires_typename: bool,
}

#[allow(unused)]
impl<'a> SelectionSet<'a> {
    pub(crate) fn fields(&self) -> impl Iterator<Item = Field<'a>> + 'a {
        self.data_fields()
            .map(Field::Data)
            .chain(self.typename_fields().map(Field::Typename))
    }

    pub(crate) fn data_fields(&self) -> impl Iterator<Item = DataField<'a>> + 'a {
        let ctx = self.ctx;
        self.item
            .data_field_ids
            .into_iter()
            .filter(|id| !self.ctx.query_modifications.skipped_data_fields[*id])
            .map(move |id| DataField { ctx, id })
    }

    pub(crate) fn typename_fields(&self) -> impl Iterator<Item = TypenameField<'a>> + 'a {
        let ctx = self.ctx;
        self.item
            .typename_field_ids
            .into_iter()
            .filter(|id| !self.ctx.query_modifications.skipped_typename_fields[*id])
            .map(move |id| TypenameField { ctx, id })
    }

    pub(crate) fn requires_typename(&self) -> bool {
        self.requires_typename
    }
}

impl std::fmt::Debug for SelectionSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionSet")
            .field("data_fields", &self.data_fields().collect::<Vec<_>>())
            .field("typename_fields", &self.typename_fields().collect::<Vec<_>>())
            .finish()
    }
}
