use serde::{Deserialize, Serialize};

use super::{TableColumnId, UniqueConstraintId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UniqueConstraintColumn {
    unique_constraint_id: UniqueConstraintId,
    column_id: TableColumnId,
}

impl UniqueConstraintColumn {
    pub fn new(unique_constraint_id: UniqueConstraintId, column_id: TableColumnId) -> Self {
        Self {
            unique_constraint_id,
            column_id,
        }
    }

    pub(crate) fn unique_constraint_id(&self) -> UniqueConstraintId {
        self.unique_constraint_id
    }

    pub(crate) fn column_id(&self) -> TableColumnId {
        self.column_id
    }
}
