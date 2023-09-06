use super::{table_column::TableColumnWalker, unique_constraint::UniqueConstraintWalker, Walker};
use crate::database_definition::{UniqueConstraintColumn, UniqueConstraintColumnId};

/// A column that is part of a unique constraint.
pub type UniqueConstraintColumnWalker<'a> = Walker<'a, UniqueConstraintColumnId>;

impl<'a> UniqueConstraintColumnWalker<'a> {
    /// The constraint this column is part of.
    pub fn constraint(self) -> UniqueConstraintWalker<'a> {
        self.walk(self.get().unique_constraint_id())
    }

    /// The column in the table this column refers to.
    pub fn table_column(self) -> TableColumnWalker<'a> {
        self.walk(self.get().column_id())
    }

    fn get(self) -> &'a UniqueConstraintColumn {
        &self.database_definition.unique_constraint_columns[self.id.0 as usize]
    }
}
