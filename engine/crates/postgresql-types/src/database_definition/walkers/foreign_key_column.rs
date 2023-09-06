use super::{foreign_key::ForeignKeyWalker, table_column::TableColumnWalker, Walker};
use crate::database_definition::{ForeignKeyColumn, ForeignKeyColumnId};

pub(crate) type ForeignKeyColumnWalker<'a> = Walker<'a, ForeignKeyColumnId>;

impl<'a> ForeignKeyColumnWalker<'a> {
    pub fn constraint(self) -> ForeignKeyWalker<'a> {
        self.walk(self.get().foreign_key_id())
    }

    pub fn constrained_column(self) -> TableColumnWalker<'a> {
        self.walk(self.get().constrained_column_id())
    }

    pub fn referenced_column(self) -> TableColumnWalker<'a> {
        self.walk(self.get().referenced_column_id())
    }

    fn get(self) -> &'a ForeignKeyColumn {
        &self.database_definition.foreign_key_columns[self.id.0 as usize]
    }
}
