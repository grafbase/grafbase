use serde::{Deserialize, Serialize};

use super::{ForeignKey, ForeignKeyId, TableId};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub(super) struct Relations {
    /// Ordered by table id
    pub(super) from: Vec<(TableId, ForeignKeyId)>,
    /// Ordered by table id
    pub(super) to: Vec<(TableId, ForeignKeyId)>,
}

impl Relations {
    pub(super) fn push_relation(&mut self, foreign_key: &ForeignKey<String>, id: ForeignKeyId) {
        self.from.push((foreign_key.constrained_table_id(), id));
        self.to.push((foreign_key.referenced_table_id(), id));
    }
}
