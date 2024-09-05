use wrapping::ListWrapping;

use crate::TypeRecord;

impl TypeRecord {
    pub fn wrapped_by(self, list_wrapping: ListWrapping) -> Self {
        Self {
            wrapping: self.wrapping.wrapped_by(list_wrapping),
            ..self
        }
    }
}
