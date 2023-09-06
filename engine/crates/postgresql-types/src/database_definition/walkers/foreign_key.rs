use super::{ForeignKeyColumnWalker, TableWalker, Walker};
use crate::database_definition::{names::StringId, ForeignKey, ForeignKeyColumnId, ForeignKeyId};

pub(crate) type ForeignKeyWalker<'a> = Walker<'a, ForeignKeyId>;

impl<'a> ForeignKeyWalker<'a> {
    pub fn name(self) -> &'a str {
        self.get_name(self.get().constraint_name())
    }

    pub fn schema(self) -> &'a str {
        &self.database_definition.schemas[self.get().schema_id().0 as usize]
    }

    pub fn columns(self) -> impl ExactSizeIterator<Item = ForeignKeyColumnWalker<'a>> {
        let range = super::range_for_key(&self.database_definition.foreign_key_columns, self.id, |column| {
            column.foreign_key_id()
        });

        range.map(move |id| self.walk(ForeignKeyColumnId(id as u32)))
    }

    pub fn constrained_table(self) -> TableWalker<'a> {
        self.walk(self.get().constrained_table_id())
    }

    pub fn referenced_table(self) -> TableWalker<'a> {
        self.walk(self.get().referenced_table_id())
    }

    pub fn all_columns_use_supported_types(self) -> bool {
        self.columns()
            .all(|column| column.constrained_column().has_supported_type())
    }

    fn get(self) -> &'a ForeignKey<StringId> {
        &self.database_definition.foreign_keys[self.id.0 as usize]
    }
}
