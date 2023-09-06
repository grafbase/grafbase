use super::{unique_constraint_column::UniqueConstraintColumnWalker, TableColumnWalker, TableWalker, Walker};
use crate::database_definition::{
    names::StringId, ConstraintType, UniqueConstraint, UniqueConstraintColumnId, UniqueConstraintId,
};

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

    /// True, if all the given columns are part of the constraint.
    pub fn has_all_the_columns(self, mut columns: impl ExactSizeIterator<Item = TableColumnWalker<'a>>) -> bool {
        columns.all(|left| self.columns().any(|right| left == right.table_column()))
    }

    /// True, if all columns in the constraint have a type we support.
    pub fn all_columns_use_supported_types(self) -> bool {
        self.columns().all(|column| column.table_column().has_supported_type())
    }

    /// True, if the constraint is the primary key of the table.
    pub fn is_primary(self) -> bool {
        matches!(self.get().r#type(), ConstraintType::Primary)
    }

    fn get(self) -> &'a UniqueConstraint<StringId> {
        &self.database_definition.unique_constraints[self.id.0 as usize]
    }
}
