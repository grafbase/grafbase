use serde::{Deserialize, Serialize};

use super::{ForeignKeyId, TableColumnId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ForeignKeyColumn {
    foreign_key_id: ForeignKeyId,
    constrained_column_id: TableColumnId,
    referenced_column_id: TableColumnId,
}

impl ForeignKeyColumn {
    pub fn new(
        foreign_key_id: ForeignKeyId,
        constrained_column_id: TableColumnId,
        referenced_column_id: TableColumnId,
    ) -> Self {
        Self {
            foreign_key_id,
            constrained_column_id,
            referenced_column_id,
        }
    }

    pub(crate) fn foreign_key_id(&self) -> ForeignKeyId {
        self.foreign_key_id
    }

    pub(crate) fn constrained_column_id(&self) -> TableColumnId {
        self.constrained_column_id
    }

    pub(crate) fn referenced_column_id(&self) -> TableColumnId {
        self.referenced_column_id
    }
}
