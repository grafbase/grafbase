use super::{unique_constraint_column::UniqueConstraintColumnWalker, TableColumnWalker, TableWalker, Walker};
use crate::database_definition::{names::StringId, UniqueConstraint, UniqueConstraintColumnId, UniqueConstraintId};

/// Defines a unique constraint in a table.
pub type UniqueConstraintWalker<'a> = Walker<'a, UniqueConstraintId>;

impl<'a> UniqueConstraintWalker<'a> {
    /// The table of this constraint.
    pub fn table(self) -> TableWalker<'a> {
        self.walk(self.get().table_id())
    }

    /// The constraint name.
    pub fn name(self) -> &'a str {
        self.get_name(self.get().name())
    }

    /// The columns defining the unique value.
    pub fn columns(self) -> impl ExactSizeIterator<Item = UniqueConstraintColumnWalker<'a>> + 'a {
        let range = super::range_for_key(&self.database_definition.unique_constraint_columns, self.id, |column| {
            column.unique_constraint_id()
        });

        range.map(move |id| self.walk(UniqueConstraintColumnId(id as u32)))
    }

    /// True, if the given columns are forming a unique constraint.
    pub fn contains_exactly_columns(self, columns: impl ExactSizeIterator<Item = TableColumnWalker<'a>>) -> bool {
        self.columns()
            .map(UniqueConstraintColumnWalker::table_column)
            .eq(columns)
    }

    /// True, if all columns in the constraint have a type we support.
    pub fn all_columns_use_supported_types(self) -> bool {
        self.columns().all(|column| column.table_column().has_supported_type())
    }

    fn get(self) -> &'a UniqueConstraint<StringId> {
        &self.database_definition.unique_constraints[self.id.0 as usize]
    }
}
