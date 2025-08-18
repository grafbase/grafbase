use walker::Walk as _;

use crate::prepare::{QueryPartition, ResponseObjectSetMetadata};

impl<'a> QueryPartition<'a> {
    pub fn input(&self) -> ResponseObjectSetMetadata<'a> {
        self.as_ref().input_id.walk(self.ctx)
    }
}
