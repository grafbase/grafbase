use crate::database_definition::{ids::BackRelationId, ForeignKeyId, TableId};

use super::{ForeignKeyWalker, TableColumnWalker, TableWalker, Walker};

/// A relation from the referenced side of a foreign key. The constraint
/// is defined on the other side.
pub type BackRelationWalker<'a> = Walker<'a, BackRelationId>;

impl<'a> BackRelationWalker<'a> {
    /// The table this relation starts from, no foreign key on this table.
    pub fn referencing_table(self) -> TableWalker<'a> {
        self.foreign_key().referenced_table()
    }

    /// The opposite table. For back-relations, the table with the foreign key.
    pub fn referenced_table(self) -> TableWalker<'a> {
        self.foreign_key().constrained_table()
    }

    /// The columns on this table that are forming the constraint.
    pub fn referencing_columns(self) -> impl ExactSizeIterator<Item = TableColumnWalker<'a>> {
        self.foreign_key().columns().map(|column| column.referenced_column())
    }

    /// The columns on the other table that are forming the constraint.
    pub fn referenced_columns(self) -> impl ExactSizeIterator<Item = TableColumnWalker<'a>> {
        self.foreign_key().columns().map(|column| column.constrained_column())
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
        self.walk(self.ids().1)
    }

    fn ids(self) -> (TableId, ForeignKeyId) {
        self.database_definition.relations.to[self.id.0 as usize]
    }
}
