use serde::{Deserialize, Serialize};

use super::{names::StringId, TableId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UniqueConstraint<T> {
    pub(super) table_id: TableId,
    pub(super) constraint_name: T,
}

impl<T> UniqueConstraint<T> {
    pub(crate) fn table_id(&self) -> TableId {
        self.table_id
    }
}

impl UniqueConstraint<String> {
    pub fn new(table_id: TableId, constraint_name: String) -> Self {
        Self {
            table_id,
            constraint_name,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.constraint_name
    }
}

impl UniqueConstraint<StringId> {
    pub(crate) fn name(&self) -> StringId {
        self.constraint_name
    }
}
