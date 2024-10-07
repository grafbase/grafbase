use super::{FederatedGraph, Field, FieldId};

impl FederatedGraph {
    pub fn push_field(&mut self, field: Field) -> FieldId {
        let id = FieldId::from(self.fields.len());
        self.fields.push(field);
        id
    }
}
