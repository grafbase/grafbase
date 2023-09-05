use crate::database_definition::{ids::ForwardRelationId, ForeignKeyId, TableId};

use super::{ForeignKeyWalker, TableColumnWalker, TableWalker, Walker};

/// A relation from the side of a foreign key. Foreign key
/// is defined from this table.
pub type ForwardRelationWalker<'a> = Walker<'a, ForwardRelationId>;

impl<'a> ForwardRelationWalker<'a> {
    /// The table this relation starts from. For forward relations, the table with the foreign key.
    pub fn referencing_table(self) -> TableWalker<'a> {
        self.foreign_key().constrained_table()
    }

    /// The opposite table, no foreign key on this table.
    pub fn referenced_table(self) -> TableWalker<'a> {
        self.foreign_key().referenced_table()
    }

    /// The columns on this table that are forming the constraint.
    pub fn referencing_columns(self) -> impl ExactSizeIterator<Item = TableColumnWalker<'a>> {
        self.foreign_key().columns().map(|column| column.constrained_column())
    }

    /// The columns on the other table that are forming the constraint.
    pub fn referenced_columns(self) -> impl ExactSizeIterator<Item = TableColumnWalker<'a>> {
        self.foreign_key().columns().map(|column| column.referenced_column())
    }

    /// True, if the referenced row is unique, this means there can only be at most one related row.
    pub fn is_referenced_row_unique(self) -> bool {
        self.referenced_table()
            .unique_constraints()
            .any(|constraint| constraint.contains_exactly_columns(self.referenced_columns()))
    }

    /// True, if all the columns of the relation are of supported type.
    pub fn all_columns_use_supported_types(self) -> bool {
        self.foreign_key().all_columns_use_supported_types()
    }

    fn foreign_key(self) -> ForeignKeyWalker<'a> {
        self.walk(self.get().1)
    }

    fn get(self) -> (TableId, ForeignKeyId) {
        self.database_definition.relations.from[self.id.0 as usize]
    }
}
