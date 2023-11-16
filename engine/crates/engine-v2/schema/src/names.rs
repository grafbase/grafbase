use crate::{FieldId, InterfaceId, ObjectId, Schema, UnionId};

/// Small abstraction over the actual names to make easier to deal with
/// renames later.
/// Argument names aren't supported given current schema design.
pub trait Names {
    fn field(&self, field_id: FieldId) -> &str;
    fn object(&self, object_id: ObjectId) -> &str;
    fn union(&self, union_id: UnionId) -> &str;
    fn interface(&self, interface_id: InterfaceId) -> &str;
}

impl Names for Schema {
    fn field(&self, field_id: FieldId) -> &str {
        &self[self[field_id].name]
    }

    fn object(&self, object_id: ObjectId) -> &str {
        &self[self[object_id].name]
    }

    fn union(&self, union_id: UnionId) -> &str {
        &self[self[union_id].name]
    }

    fn interface(&self, interface_id: InterfaceId) -> &str {
        &self[self[interface_id].name]
    }
}
