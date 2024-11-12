use super::{FederatedGraph, Object as ObjectRecord, ObjectId};

impl FederatedGraph {
    pub fn push_object(&mut self, object: ObjectRecord) -> ObjectId {
        let id = self.objects.len().into();
        self.objects.push(object);
        id
    }
}
